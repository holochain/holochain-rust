use holochain_metrics::{cloudwatch::*, stats::Stats, *};
use std::{iter::FromIterator, time::*};
fn main() {
    println!("WARNING: This requires `source ~/deploy/assume-role.sh` to be run first!");
    let mut cloudwatch = CloudWatchLogger::default();

    let latency = Metric::new("latency", 100.0);
    cloudwatch.publish(&latency);

    let now = SystemTime::now();
    let query = cloudwatch.query(&UNIX_EPOCH, &now);

    println!("query: {:?}", query);
    let metrics = CloudWatchLogger::metrics_of_query(query);
    let vec = Vec::from_iter(metrics);
    println!("metrics: {:?}", vec);

    let stats = Stats::from_iter(vec.into_iter());
    println!("stats: {:?}", stats);
}
