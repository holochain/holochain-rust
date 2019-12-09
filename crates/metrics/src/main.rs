extern crate structopt;
use crate::structopt::StructOpt;
use holochain_metrics::{
    cloudwatch::*,
    stats::{StatCheck, StatsByMetric, StatsRecord},
    *,
};
use rusoto_core::Region;
use std::{
    fs::File,
    io::{BufReader, BufWriter},
};

fn enable_logging() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "debug");
    }
    let _ = env_logger::builder()
        .default_format_timestamp(false)
        .default_format_module_path(false)
        .is_test(true)
        .try_init();
}

#[derive(StructOpt)]
#[structopt(name = "metrics", about = "Holochain metric utilities")]
enum Command {
    PrintCloudwatchStats {
        #[structopt(
            name = "region",
            short = "r",
            help = "The AWS region, defaults to eu-central-1."
        )]
        region: Option<Region>,
        #[structopt(
            name = "log_group_name",
            short = "l",
            help = "The AWS log group name to query over."
        )]
        log_group_name: Option<String>,
        #[structopt(
            name = "assume_role_arn",
            short = "a",
            help = "Optional override for the amazon role to assume when querying"
        )]
        assume_role_arn: Option<String>,
        #[structopt(flatten)]
        query_args: QueryArgs,
    },

    #[structopt(
        name = "print-log-stats",
        help = "Prints descriptive stats in csv form over a time range"
    )]
    PrintLogStats {
        #[structopt(name = "log_file", short = "f")]
        log_file: String,
    },
    #[structopt(
        name = "print-stat-check",
        help = "Prints descriptive stats in csv form over a time range"
    )]
    StatCheck {
        #[structopt(name = "expected_csv_file", short = "ef")]
        expected_csv_file: String,
        #[structopt(name = "acual_csv_file", short = "af")]
        actual_csv_file: String,
        #[structopt(name = "result_csv_file", short = "rf")]
        result_csv_file: String,
    },
}

fn setup_aws_env() {
    // HACK fix an issue with cloudwatch logs api
    std::env::set_var("AWS_REGION", "eu-central-1")
}

fn main() {
    enable_logging();
    setup_aws_env();

    let command = Command::from_args();

    match command {
        Command::PrintCloudwatchStats {
            region,
            log_group_name,
            query_args,
            assume_role_arn,
        } => {
            let region = region.unwrap_or_default();
            let log_group_name = log_group_name.unwrap_or_else(CloudWatchLogger::default_log_group);
            let assume_role_arn = assume_role_arn
                .unwrap_or_else(|| crate::cloudwatch::FINAL_EXAM_NODE_ROLE.to_string());
            print_cloudwatch_stats(&query_args, log_group_name, &region, &assume_role_arn);
        }
        Command::PrintLogStats { log_file } => print_log_stats(log_file),
        Command::StatCheck {
            expected_csv_file,
            actual_csv_file,
            result_csv_file,
        } => print_stat_check(expected_csv_file, actual_csv_file, result_csv_file),
    }
}

fn print_cloudwatch_stats(
    query_args: &QueryArgs,
    log_group_name: String,
    region: &Region,
    assume_role_arn: &str,
) {
    let cloudwatch = CloudWatchLogger::with_log_group(
        log_group_name,
        crate::cloudwatch::assume_role(&region, assume_role_arn),
        region,
    );

    let stats: StatsByMetric<_> = cloudwatch.query_and_aggregate(query_args);

    stats.write_csv(std::io::stdout()).unwrap();
}

fn print_log_stats(log_file: String) {
    let metrics = crate::logger::metrics_from_file(log_file.clone()).unwrap();
    let stats = StatsByMetric::from_iter_with_stream_id(metrics, log_file);
    stats.write_csv(std::io::stdout()).unwrap()
}

/// Prints to stdout human readonly pass/fail info
/// Saves to `result_csv_file` gradient info
fn print_stat_check(
    expected_csv_file: String, // StatsByMetric
    actual_csv_file: String,   // StatsByMetric
    result_csv_file: String,   // A collection of CheckedStatRecords
) {
    let mut actual_reader = BufReader::new(File::open(actual_csv_file).unwrap());
    let mut expected_reader = BufReader::new(File::open(expected_csv_file).unwrap());
    let actual_csv_data = StatsByMetric::<StatsRecord>::from_reader(&mut actual_reader).unwrap();
    let expected_csv_data =
        StatsByMetric::<StatsRecord>::from_reader(&mut expected_reader).unwrap();

    let checked =
        crate::stats::LessThanStatCheck::default().check_all(&expected_csv_data, &actual_csv_data);

    let file = BufWriter::new(File::create(result_csv_file).unwrap());
    let mut writer = csv::Writer::from_writer(file);
    for (key, record) in checked.iter() {
        match record {
            Ok(record) => {
                writer.serialize(record).unwrap();
            }
            Err((record, errors)) => {
                writer.serialize(record).unwrap();
                for e in errors {
                    println!("{}: {}", key, e);
                }
            }
        }
    }
    writer.flush().unwrap();
}
