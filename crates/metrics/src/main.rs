extern crate structopt;

use crate::structopt::StructOpt;
use holochain_metrics::{cloudwatch::*, stats::StatsByMetric, *};
use rusoto_core::Region;
use std::{iter::FromIterator, time::*};

#[derive(StructOpt)]
#[structopt(name = "metrics", about = "Holochain metric utilities")]
enum Command {
    #[structopt(
        name = "cloudwatch-test",
        about = "Runs a simple smoke test of cloudwatch publisig features"
    )]
    CloudwatchTest,
    #[structopt(
        name = "print-stats",
        about = "Prints descriptive stats in csv form over a time range"
    )]
    PrintStats {
        #[structopt(name = "region", short = "r")]
        region: Option<Region>,
        #[structopt(name = "log_group_name", short = "l")]
        log_group_name: Option<String>,
        #[structopt(name = "start-time")]
        start_time: u64,
        #[structopt(name = "stop-time")]
        stop_time: u64,
    },
}

fn main() {
    let command = Command::from_args();

    match command {
        Command::CloudwatchTest => cloudwatch_test(),
        Command::PrintStats {
            region,
            log_group_name,
            start_time,
            stop_time,
        } => {
            let region = region.unwrap_or_default();
            let log_group_name = log_group_name.unwrap_or_else(CloudWatchLogger::default_log_group);
            let start_time = UNIX_EPOCH
                .checked_add(Duration::from_secs(start_time))
                .unwrap();
            let stop_time = UNIX_EPOCH
                .checked_add(Duration::from_secs(stop_time))
                .unwrap();
            print_stats(&start_time, &stop_time, log_group_name, &region)
        }
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

    let now = SystemTime::now();
    let query = cloudwatch.query(&UNIX_EPOCH, &now);

    println!("query: {:?}", query);
    let metrics = CloudWatchLogger::metrics_of_query(query);
    let vec = Vec::from_iter(metrics);
    println!("metrics: {:?}", vec);

    let stats = StatsByMetric::from_iter(vec.into_iter());
    println!("stats: {:?}", stats);

    stats.print_csv().unwrap()
}

fn print_stats(
    start_time: &SystemTime,
    stop_time: &SystemTime,
    log_group_name: String,
    region: &Region,
) {
    let cloudwatch = CloudWatchLogger::with_log_group(log_group_name, region);

    let stats: StatsByMetric = cloudwatch.query_and_aggregate(start_time, stop_time);

    stats.print_csv().unwrap()
}
