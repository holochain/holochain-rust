/// Provides statistical features over metric data.
use crate::Metric;
use num_traits::float::Float;
/// Extends the metric api with statistical aggregation functions
use stats::{Commute, OnlineStats};
use std::{collections::HashMap, iter::FromIterator};

/// An extension of `OnlineStats` that also incrementally tracks
/// max and min values.
#[derive(Debug, Clone)]
pub struct DescriptiveStats {
    online_stats: OnlineStats,
    max: f64,
    min: f64,
    cnt: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatsRecord {
    pub name: Option<String>,
    pub max: f64,
    pub min: f64,
    pub cnt: u64,
    pub mean: f64,
    pub variance: f64,
    pub stddev: f64,
}

impl StatsRecord {
    pub fn new<S: Into<String>>(metric_name: S, desc: DescriptiveStats) -> Self {
        let metric_name = metric_name.into();
        let mut record: Self = desc.into();
        record.name = Some(metric_name);
        record
    }
}

impl From<DescriptiveStats> for StatsRecord {
    fn from(desc_stats: DescriptiveStats) -> Self {
        Self {
            name: None,
            max: desc_stats.max(),
            min: desc_stats.min(),
            stddev: desc_stats.stddev(),
            mean: desc_stats.mean(),
            variance: desc_stats.variance(),
            cnt: desc_stats.count(),
        }
    }
}

// TODO is this necessary?
impl Copy for DescriptiveStats {}

impl DescriptiveStats {
    /// An initial empty statistic.
    pub fn empty() -> Self {
        Self {
            online_stats: OnlineStats::new(),
            max: f64::min_value(),
            min: f64::max_value(),
            cnt: 0,
        }
    }

    /// Adds a value to the running statistic.
    pub fn add(&mut self, value: f64) {
        self.online_stats.add(value);
        if value > self.max {
            self.max = value
        }
        if value < self.min {
            self.min = value
        }
        self.cnt += 1;
    }

    /// The mean value of the running statistic.
    pub fn mean(&self) -> f64 {
        self.online_stats.mean()
    }

    /// The standard deviation of the running statistic.
    pub fn stddev(&self) -> f64 {
        self.online_stats.stddev()
    }

    /// The variance of the running statistic.
    pub fn variance(&self) -> f64 {
        self.online_stats.variance()
    }

    /// The max of the running statistic.
    pub fn max(&self) -> f64 {
        self.max
    }

    /// The min of the running statistic.
    pub fn min(&self) -> f64 {
        self.min
    }

    /// The number of samples of the running statistic.
    pub fn count(&self) -> u64 {
        self.cnt
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
        self.cnt += rhs.cnt;
    }
}

/// All combined descriptive statistics mapped by name of the metric
#[derive(Shrinkwrap, Debug, Clone)]
pub struct StatsByMetric(HashMap<String, DescriptiveStats>);

impl StatsByMetric {
    pub fn to_records(&self) -> Box<dyn Iterator<Item = StatsRecord>> {
        let me = self.0.clone();
        Box::new(
            me.into_iter()
                .map(|(name, stat)| StatsRecord::new(name, stat)),
        )
    }

    pub fn print_csv(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut writer = csv::Writer::from_writer(std::io::stdout());
        let records = self.to_records();
        for record in records {
            writer.serialize(record)?;
        }
        writer.flush()?;
        Ok(())
    }
}

impl FromIterator<Metric> for StatsByMetric {
    fn from_iter<I: IntoIterator<Item = Metric>>(source: I) -> StatsByMetric {
        StatsByMetric(source.into_iter().fold(
            HashMap::new(),
            |mut stats_by_metric_name, metric| {
                let entry = stats_by_metric_name.entry(metric.name);

                let online_stats = entry.or_insert_with(DescriptiveStats::empty);
                online_stats.add(metric.value);
                stats_by_metric_name
            },
        ))
    }
}

impl Commute for StatsByMetric {
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
        let stats = StatsByMetric::from_iter(all_data);

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
