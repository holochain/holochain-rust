use holochain_metrics::{cloudwatch::*, stats::StatsByMetric, *};
use std::{iter::FromIterator, time::*};
fn main() {
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
