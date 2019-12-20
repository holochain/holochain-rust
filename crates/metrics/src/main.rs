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
    path::PathBuf,
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
    #[structopt(
        name = "print-cloudwatch-stats",
        about = "Prints descriptive stats in csv format over a time range in cloudwatch"
    )]
    PrintCloudwatchStats {
        #[structopt(flatten)]
        cloudwatch_options: CloudwatchLogsOptions,

        #[structopt(
            name = "aggregation_pattern",
            long = "aggregation_pattern",
            short = "g"
        )]
        aggregation_pattern: Option<String>,
    },
    #[structopt(
        name = "print-cloudwatch-metrics",
        about = "Prints the metrics for a cloudwatch query in csv format"
    )]
    PrintCloudwatchMetrics(CloudwatchLogsOptions),
    #[structopt(
        name = "print-log-metrics",
        about = "Prints metrics in csv format over a log file"
    )]
    PrintLogMetrics {
        #[structopt(name = "log_file", short = "f")]
        log_file: PathBuf,
    },
    #[structopt(
        name = "print-metric-stats",
        about = "Prints descriptive stats in csv format over a metric csv file"
    )]
    PrintMetricStats {
        #[structopt(name = "csv_file", short = "f")]
        csv_file: PathBuf,
        #[structopt(
            name = "aggregation_pattern",
            long = "aggregation-pattern",
            short = "g"
        )]
        aggregation_pattern: Option<String>,
    },
    #[structopt(
        name = "print-log-stats",
        about = "Prints descriptive stats in csv format over a log file"
    )]
    PrintLogStats {
        #[structopt(name = "log_file", short = "f")]
        log_file: PathBuf,
        #[structopt(
            name = "aggregation_pattern",
            long = "aggregation-pattern",
            short = "g"
        )]
        aggregation_pattern: Option<String>,
    },
    #[structopt(
        name = "print-stat-check",
        about = "Prints stat checks and save results in csv format"
    )]
    StatCheck {
        #[structopt(name = "expected_csv_file", short = "ef")]
        expected_csv_file: PathBuf,
        #[structopt(name = "acual_csv_file", short = "af")]
        actual_csv_file: PathBuf,
        #[structopt(name = "result_csv_file", short = "rf")]
        result_csv_file: PathBuf,
    },
}

fn setup_aws_env() {
    // HACK fix an issue with cloudwatch logs api
    if std::env::var("AWS_REGION").is_err() {
        std::env::set_var("AWS_REGION", "eu-central-1")
    }
}

fn main() {
    enable_logging();
    setup_aws_env();

    let command = Command::from_args();

    match command {
        Command::PrintCloudwatchMetrics(CloudwatchLogsOptions {
            region,
            log_group_name,
            query_args,
            assume_role_arn,
        }) => {
            let region = region.unwrap_or_default();
            let log_group_name = log_group_name.unwrap_or_else(CloudWatchLogger::default_log_group);
            let assume_role_arn = assume_role_arn
                .unwrap_or_else(|| crate::cloudwatch::FINAL_EXAM_NODE_ROLE.to_string());
            print_cloudwatch_metrics(&query_args, log_group_name, &region, &assume_role_arn);
        }
        Command::PrintCloudwatchStats {
            cloudwatch_options:
                CloudwatchLogsOptions {
                    region,
                    log_group_name,
                    query_args,
                    assume_role_arn,
                },
            aggregation_pattern,
        } => {
            let region = region.unwrap_or_default();
            let log_group_name = log_group_name.unwrap_or_else(CloudWatchLogger::default_log_group);
            let assume_role_arn = assume_role_arn
                .unwrap_or_else(|| crate::cloudwatch::FINAL_EXAM_NODE_ROLE.to_string());
            print_cloudwatch_stats(
                &query_args,
                log_group_name,
                &region,
                &assume_role_arn,
                aggregation_pattern,
            );
        }
        Command::PrintLogStats {
            log_file,
            aggregation_pattern,
        } => print_log_stats(log_file, aggregation_pattern),
        Command::PrintMetricStats {
            csv_file,
            aggregation_pattern,
        } => print_metric_stats(csv_file, aggregation_pattern),
        Command::PrintLogMetrics { log_file } => print_log_metrics(log_file),
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
    _aggregation_pattern: Option<String>,
) {
    let cloudwatch = CloudWatchLogger::with_log_group(
        log_group_name,
        crate::cloudwatch::assume_role(&region, assume_role_arn),
        region,
    );

    let stats: StatsByMetric<_> = cloudwatch.query_and_aggregate(query_args);

    stats.write_csv(std::io::stdout()).unwrap();
}

fn print_cloudwatch_metrics(
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

    let metrics = cloudwatch.query_metrics(query_args);
    let file = BufWriter::new(std::io::stdout());
    let mut writer = csv::Writer::from_writer(file);

    for m in metrics {
        writer.serialize(m).unwrap();
    }
    writer.flush().unwrap();
}

// TODO use aggregation patern
fn print_log_stats(log_file: PathBuf, _aggregation_pattern: Option<String>) {
    let metrics = crate::logger::metrics_from_file(log_file.clone()).unwrap();
    let stats = StatsByMetric::from_iter_with_stream_id(
        metrics,
        log_file.to_str().unwrap_or_else(|| "unknown"),
        //        aggregation_pattern
    );
    stats.write_csv(std::io::stdout()).unwrap()
}

fn print_log_metrics(log_file: PathBuf) {
    let metrics = crate::logger::metrics_from_file(log_file).unwrap();

    let file = BufWriter::new(std::io::stdout());
    let mut writer = csv::Writer::from_writer(file);

    for m in metrics {
        writer.serialize(m).unwrap();
    }
    writer.flush().unwrap();
}

/// Prints to stdout human readonly pass/fail info
/// Saves to `result_csv_file` gradient info
fn print_stat_check(
    expected_csv_file: PathBuf, // StatsByMetric
    actual_csv_file: PathBuf,   // StatsByMetric
    result_csv_file: PathBuf,   // A collection of CheckedStatRecords
) {
    let mut actual_reader = BufReader::new(File::open(actual_csv_file.clone()).unwrap());
    let actual_csv_data = StatsByMetric::<StatsRecord>::from_reader(&mut actual_reader).unwrap();

    let expected_csv_data = File::open(expected_csv_file.clone()).map(|expected_csv_reader| {
        let mut expected_reader = BufReader::new(expected_csv_reader);
        let expected_csv_data =
            StatsByMetric::<StatsRecord>::from_reader(&mut expected_reader).unwrap();
        expected_csv_data
    }).unwrap_or_else(|e| {
        println!("Expected data not found for path {:?} because error {:?}, bootstrapping from actual {:?}",
            expected_csv_file.clone(), e, actual_csv_file);
        std::fs::copy(actual_csv_file.clone(), expected_csv_file).unwrap();
        actual_csv_data.clone()
    });

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
                println!("{}", key);
                for e in errors {
                    println!("\t{}", e);
                }
            }
        }
    }
    writer.flush().unwrap();
}

fn print_metric_stats(csv_file: PathBuf, aggregation_pattern: Option<String>) {
    let reader = BufReader::new(File::open(csv_file).unwrap());
    let mut reader = csv::Reader::from_reader(reader);
    let metrics = crate::metrics_from_reader!(reader);
    let aggregation_pattern = aggregation_pattern.unwrap_or_else(|| "([.]*)".into());
    let re = regex::Regex::new(aggregation_pattern.as_str()).unwrap();
    let stats_by_metric = StatsByMetric::group_by_regex(&re, metrics);
    stats_by_metric.write_csv(std::io::stdout()).unwrap();
}
