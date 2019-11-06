use crate::Metric;
/// Extends the metric api with statistical aggregation functions
use stats::{Commute, OnlineStats};
use std::{collections::HashMap, iter::FromIterator};

use num_traits::float::Float;

#[derive(Debug, Clone)]
pub struct DescriptiveStats {
    online_stats: OnlineStats,
    max: f64,
    min: f64,
}

impl Copy for DescriptiveStats {}

impl DescriptiveStats {
    pub fn empty() -> Self {
        Self {
            online_stats: OnlineStats::new(),
            max: f64::min_value(),
            min: f64::max_value(),
        }
    }

    pub fn add(&mut self, value: f64) {
        self.online_stats.add(value);
        if value > self.max {
            self.max = value
        }
        if value < self.min {
            self.min = value
        }
    }

    pub fn mean(&self) -> f64 {
        self.online_stats.mean()
    }

    pub fn stddev(&self) -> f64 {
        self.online_stats.stddev()
    }

    pub fn variance(&self) -> f64 {
        self.online_stats.variance()
    }

    pub fn max(&self) -> f64 {
        self.max
    }

    pub fn min(&self) -> f64 {
        self.min
    }
}

impl Commute for DescriptiveStats {
    fn merge(&mut self, rhs: Self) {
        self.online_stats.merge(rhs.online_stats);
        if rhs.max > self.max {
            self.max = rhs.max
        }
        if rhs.min < self.min {
            self.min = rhs.min
        }
    }
}

#[derive(Shrinkwrap, Debug, Clone)]
pub struct Stats(HashMap<String, DescriptiveStats>);

impl FromIterator<Metric> for Stats {
    fn from_iter<I: IntoIterator<Item = Metric>>(source: I) -> Stats {
        Stats(
            source
                .into_iter()
                .fold(HashMap::new(), |mut stats_by_metric_name, metric| {
                    let entry = stats_by_metric_name.entry(metric.name);

                    let online_stats = entry.or_insert_with(DescriptiveStats::empty);
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
            let online_stats = entry.or_insert_with(DescriptiveStats::empty);
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

        assert_eq!(latency_stats.min(), 50.0);
        assert_eq!(latency_stats.max(), 150.0);

        assert_eq!(size_stats.min(), 1.0);
        assert_eq!(size_stats.max(), 100.0);
    }

}
