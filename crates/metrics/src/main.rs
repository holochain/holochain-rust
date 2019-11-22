extern crate structopt;
use crate::structopt::StructOpt;
use holochain_metrics::{cloudwatch::*, stats::StatsByMetric, *};
use rusoto_core::Region;
use std::iter::FromIterator;

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
        name = "cloudwatch-test",
        about = "Runs a simple smoke test of cloudwatch publishing features"
    )]
    CloudwatchTest,
    #[structopt(
        name = "print-cloudwatch-stats",
        about = "Prints descriptive stats in csv form over a time range from a cloudwatch datasource"
    )]
    PrintCloudwatchStats {
        #[structopt(
            name = "region",
            short = "r",
            about = "The AWS region, defaults to eu-central-1."
        )]
        region: Option<Region>,
        #[structopt(
            name = "log_group_name",
            short = "l",
            about = "The AWS log group name to query over."
        )]
        log_group_name: Option<String>,
        #[structopt(
            name = "assume_role_arn",
            short = "a",
            about = "Optional override for the amazon role to assume when querying"
        )]
        assume_role_arn: Option<String>,
       #[structopt(flatten)]
        query_args: QueryArgs,
    },

    #[structopt(
        name = "print-log-stats",
        about = "Prints descriptive stats in csv form over a time range"
    )]
    PrintLogStats {
        #[structopt(name = "log_file", short = "f")]
        log_file: String,
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
        Command::CloudwatchTest => cloudwatch_test(),
        Command::PrintCloudwatchStats {
            region,
            log_group_name,
            query_args,
            assume_role_arn
        } => {
            let region = region.unwrap_or_default();
            let log_group_name = log_group_name.unwrap_or_else(CloudWatchLogger::default_log_group);
            let assume_role_arn = assume_role_arn.unwrap_or_else(|| crate::cloudwatch::FINAL_EXAM_NODE_ROLE.to_string());
            print_cloudwatch_stats(&query_args, log_group_name, &region, &assume_role_arn);
        }
        Command::PrintLogStats { log_file } => print_log_stats(log_file),
    }
}

fn cloudwatch_test() {
    let mut cloudwatch = CloudWatchLogger::default();

    let latency = Metric::new("latency", 100.0);
    cloudwatch.publish(&latency);
    let latency = Metric::new("latency", 200.0);
    cloudwatch.publish(&latency);

    let size = Metric::new("size", 1000.0);
    cloudwatch.publish(&size);

    let size = Metric::new("size", 1.0);
    cloudwatch.publish(&size);

    let query = cloudwatch.query(&Default::default());

    println!("query: {:?}", query);
    let metrics = CloudWatchLogger::metrics_of_query(query);
    let vec = Vec::from_iter(metrics);
    println!("metrics: {:?}", vec);

    let stats = StatsByMetric::from_iter(vec.into_iter());
    println!("stats: {:?}", stats);

    stats.print_csv().unwrap()
}

fn print_cloudwatch_stats(query_args: &QueryArgs, log_group_name: String, region: &Region, assume_role_arn: &str) {
    let cloudwatch = CloudWatchLogger::with_log_group(
        log_group_name,
        crate::cloudwatch::assume_role(&region, assume_role_arn),
        region,
    );

    let stats: StatsByMetric = cloudwatch.query_and_aggregate(query_args);

    stats.print_csv().unwrap()
}

fn print_log_stats(log_file: String) {
    let metrics = crate::logger::metrics_from_file(log_file).unwrap();
    let stats = StatsByMetric::from_iter(metrics);
    stats.print_csv().unwrap()
}
