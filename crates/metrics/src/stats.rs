use crate::Metric;
/// Extends the metric api with statistical aggregation functions
use stats::{Commute, OnlineStats};
use std::{collections::HashMap, iter::FromIterator};

#[derive(Shrinkwrap, Debug)]
pub struct Stats(HashMap<String, OnlineStats>);

fn empty_stat() -> OnlineStats {
    OnlineStats::new()
}

impl FromIterator<Metric> for Stats {
    fn from_iter<I: IntoIterator<Item = Metric>>(source: I) -> Stats {
        Stats(
            source
                .into_iter()
                .fold(HashMap::new(), |mut stats_by_metric_name, metric| {
                    let entry = stats_by_metric_name.entry(metric.name);

                    let online_stats = entry.or_insert_with(empty_stat);
                    online_stats.add(metric.value);
                    stats_by_metric_name
                }),
        )
    }
}

impl Commute for Stats {
    fn merge(&mut self, rhs: Self) {
        for (metric_name, online_stats_rhs) in rhs.iter() {
            let entry = self.0.entry(metric_name.to_string());
            let online_stats = entry.or_insert_with(empty_stat);
            online_stats.merge(*online_stats_rhs);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn can_aggregate_stats_from_iterator() {
        let latency_data = vec![50.0, 100.0, 150.0]
            .into_iter()
            .map(|x| Metric::new("latency", x));
        let size_data = vec![1.0, 10.0, 100.0]
            .into_iter()
            .map(|x| Metric::new("size", x));
        let all_data = latency_data.chain(size_data);
        let stats = Stats::from_iter(all_data);

        let latency_stats = stats.get("latency").expect("latency stats to be present");

        assert_eq!(latency_stats.mean(), 100.0);
        let size_stats = stats.get("size").expect("size stats to be present");

        assert_eq!(size_stats.mean(), 37.0);
    }

}
