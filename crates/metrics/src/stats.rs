/// Provides statistical features over metric data.
/// Extends the metric api with statistical aggregation functions
use crate::{metrics::Metric, NEW_RELIC_LICENSE_KEY};
use num_traits::float::Float;
use stats::Commute;
use std::{
    collections::HashMap,
    error::Error,
    fmt::{self, Display, Formatter},
    io,
    iter::FromIterator,
};

use regex::Regex;

/// Generic representation of descriptive statistics.
pub trait DescriptiveStats {
    fn max(&self) -> f64;
    fn min(&self) -> f64;
    fn cnt(&self) -> f64;
    fn mean(&self) -> f64;
    fn stddev(&self) -> f64;
    fn variance(&self) -> f64;

    /// Computes percent change between two descriptive statistics
    fn percent_change(&self, other: &dyn DescriptiveStats) -> StatsRecord {
        StatsRecord {
            mean: percent_change(self.mean(), other.mean()),
            max: percent_change(self.max(), other.max()),
            min: percent_change(self.min(), other.min()),
            cnt: percent_change(self.cnt(), other.cnt()),
            stddev: percent_change(self.stddev(), other.stddev()),
            variance: percent_change(self.variance(), other.variance()),
            ..Default::default()
        }
    }
}

/// An extension of `stats::OnlineStats` that also incrementally tracks
/// max and min values.
#[derive(Debug, Clone, Shrinkwrap)]
pub struct OnlineStats {
    #[shrinkwrap(main_field)]
    online_stats: stats::OnlineStats,
    max: f64,
    min: f64,
    cnt: u64,
}

/// A statistical record, useful for serialization and display purposes.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatsRecord {
    pub metric: Option<String>,
    pub stream_id: Option<String>,
    pub max: f64,
    pub min: f64,
    pub cnt: f64,
    pub mean: f64,
    pub variance: f64,
    pub stddev: f64,
}

/// A checked statistical record to indicate differences between two statistics.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CheckedStatsRecord {
    metric: String,
    stream_id: String,
    pub expected_max: f64,
    pub expected_min: f64,
    pub expected_cnt: f64,
    pub expected_mean: f64,
    pub expected_variance: f64,
    pub expected_stddev: f64,
    pub actual_max: f64,
    pub actual_min: f64,
    pub actual_cnt: f64,
    pub actual_mean: f64,
    pub actual_variance: f64,
    pub actual_stddev: f64,
    pub percent_change_max: f64,
    pub percent_change_min: f64,
    pub percent_change_cnt: f64,
    pub percent_change_mean: f64,
    pub percent_change_variance: f64,
    pub percent_change_stddev: f64,
    pub percent_change_allowed: f64,
    pub passed: bool,
}

impl CheckedStatsRecord {
    pub fn new(
        expected: &StatsRecord,
        actual: &dyn DescriptiveStats,
        percent_change_allowed: f64,
        passed: bool,
    ) -> Self {
        let percent_change = expected.percent_change(actual);
        let metric = expected.metric.clone().unwrap_or_default();
        let stream_id = expected.stream_id.clone().unwrap_or_default();
        Self {
            metric,
            stream_id,
            expected_max: expected.max(),
            expected_min: expected.min(),
            expected_cnt: expected.cnt(),
            expected_mean: expected.mean(),
            expected_variance: expected.variance(),
            expected_stddev: expected.stddev(),
            actual_max: actual.max(),
            actual_min: actual.min(),
            actual_cnt: actual.cnt(),
            actual_mean: actual.mean(),
            actual_variance: actual.variance(),
            actual_stddev: actual.stddev(),
            percent_change_max: percent_change.max(),
            percent_change_min: percent_change.min(),
            percent_change_cnt: percent_change.cnt(),
            percent_change_mean: percent_change.mean(),
            percent_change_variance: percent_change.variance(),
            percent_change_stddev: percent_change.stddev(),
            percent_change_allowed,
            passed,
        }
    }
}

impl Default for StatsRecord {
    fn default() -> Self {
        Self {
            metric: None,
            stream_id: None,
            max: f64::min_value(),
            min: f64::max_value(),
            mean: 0.0,
            variance: 0.0,
            stddev: 0.0,
            cnt: 0.,
        }
    }
}

impl StatsRecord {
    pub fn new<S: Into<Option<String>>, S2: Into<Option<String>>, D: DescriptiveStats>(
        stream_id: S,
        metric: S2,
        desc: D,
    ) -> Self {
        let metric = metric.into();
        let stream_id = stream_id.into();
        Self {
            metric,
            stream_id,
            max: desc.max(),
            min: desc.min(),
            mean: desc.mean(),
            cnt: desc.cnt() as f64,
            stddev: desc.stddev(),
            variance: desc.variance(),
        }
    }

    /// Produces a hash map of descriptive statistics from `read` keyed by metric name.
    pub fn from_reader(
        read: &mut dyn std::io::Read,
    ) -> Result<HashMap<String, Box<dyn DescriptiveStats>>, Box<dyn Error>> {
        let mut reader = csv::Reader::from_reader(read);

        let mut stats_by_metric_name: HashMap<String, Box<dyn DescriptiveStats>> = HashMap::new();
        for record in reader.deserialize() {
            let stat: StatsRecord = record?;
            let stat_name = stat.metric.clone().map(|x| Ok(x)).unwrap_or_else(|| {
                Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "No stat name in stat record",
                )))
            })?;
            stats_by_metric_name.insert(stat_name.to_string(), Box::new(stat));
        }
        Ok(stats_by_metric_name)
    }
}

impl From<OnlineStats> for StatsRecord {
    fn from(desc_stats: OnlineStats) -> Self {
        Self {
            metric: None,
            stream_id: None,
            max: desc_stats.max(),
            min: desc_stats.min(),
            stddev: desc_stats.stddev(),
            mean: desc_stats.mean(),
            variance: desc_stats.variance(),
            cnt: desc_stats.cnt() as f64,
        }
    }
}

impl DescriptiveStats for StatsRecord {
    fn max(&self) -> f64 {
        self.max
    }
    fn min(&self) -> f64 {
        self.min
    }
    fn cnt(&self) -> f64 {
        self.cnt
    }
    fn variance(&self) -> f64 {
        self.variance
    }
    fn stddev(&self) -> f64 {
        self.stddev
    }
    fn mean(&self) -> f64 {
        self.mean
    }
}

impl Copy for OnlineStats {}

#[derive(Clone, Debug, Serialize)]
pub enum DescriptiveStatType {
    Mean,
    Max,
    Min,
    StdDev,
    Count,
}

impl Display for DescriptiveStatType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Represents a checked statistic that deviated too far.
#[derive(Clone, Debug, Serialize)]
pub struct StatFailure {
    expected: f64,
    actual: f64,
    stat_type: DescriptiveStatType,
}

impl std::fmt::Display for StatFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: Expected {}, Actual was {}",
            self.stat_type, self.expected, self.actual
        )
    }
}

impl OnlineStats {
    /// An initial empty statistic.
    pub fn empty() -> Self {
        Self {
            online_stats: stats::OnlineStats::new(),
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
}

impl DescriptiveStats for OnlineStats {
    /// The mean value of the running statistic.
    fn mean(&self) -> f64 {
        self.online_stats.mean()
    }

    /// The standard deviation of the running statistic.
    fn stddev(&self) -> f64 {
        self.online_stats.stddev()
    }

    /// The variance of the running statistic.
    fn variance(&self) -> f64 {
        self.online_stats.variance()
    }

    /// The max of the running statistic.
    fn max(&self) -> f64 {
        self.max
    }

    /// The min of the running statistic.
    fn min(&self) -> f64 {
        self.min
    }

    /// The number of samples of the running statistic.
    fn cnt(&self) -> f64 {
        self.cnt as f64
    }
}

pub trait StatCheck {
    fn check(
        &self,
        expected: &StatsRecord,
        actual: &dyn DescriptiveStats,
    ) -> Result<CheckedStatsRecord, (CheckedStatsRecord, Vec<StatFailure>)>;

    fn check_all(
        &self,
        expected: &StatsByMetric<StatsRecord>,
        actual: &StatsByMetric<StatsRecord>,
    ) -> StatCheckResult {
        StatCheckResult(HashMap::from_iter(expected.iter().map(
            |(grouping_key, expected_stat)| {
                let result = if let Some(actual_stat) = actual.get(grouping_key) {
                    self.check(expected_stat, actual_stat)
                } else {
                    self.check(expected_stat, &StatsRecord::default())
                };
                (grouping_key.clone(), result)
            },
        )))
    }
}

/// Computes percentage change between expected and actual
/// May produce `NaN`
pub fn percent_change<N: Into<f64>>(expected: N, actual: N) -> f64 {
    let e = expected.into();
    let a = actual.into();
    f64::abs(e - a) / e
}

#[derive(Clone, Debug)]
pub struct LessThanStatCheck {
    percent_change_allowed: f64,
}

impl LessThanStatCheck {}

impl Default for LessThanStatCheck {
    fn default() -> Self {
        LessThanStatCheck {
            percent_change_allowed: 0.05,
        }
    }
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_METRICS)]
impl StatCheck for LessThanStatCheck {
    fn check(
        &self,
        expected: &StatsRecord,
        actual: &dyn DescriptiveStats,
    ) -> Result<CheckedStatsRecord, (CheckedStatsRecord, Vec<StatFailure>)> {
        let percent_change = expected.percent_change(actual);

        let mut failures = Vec::new();

        let mut checked_stats_record =
            CheckedStatsRecord::new(expected, actual, self.percent_change_allowed, false);
        if percent_change.mean() > self.percent_change_allowed {
            failures.push(StatFailure {
                expected: expected.mean(),
                actual: actual.mean(),
                stat_type: DescriptiveStatType::Mean,
            })
        }

        if percent_change.stddev() > self.percent_change_allowed {
            failures.push(StatFailure {
                expected: expected.stddev(),
                actual: actual.stddev(),
                stat_type: DescriptiveStatType::StdDev,
            })
        }

        if percent_change.max() > self.percent_change_allowed {
            failures.push(StatFailure {
                expected: expected.max(),
                actual: actual.max(),
                stat_type: DescriptiveStatType::Max,
            })
        }

        if percent_change.min() > self.percent_change_allowed {
            failures.push(StatFailure {
                expected: expected.min(),
                actual: actual.min(),
                stat_type: DescriptiveStatType::Min,
            })
        }

        if percent_change.cnt() > self.percent_change_allowed {
            failures.push(StatFailure {
                expected: expected.cnt() as f64,
                actual: actual.cnt() as f64,
                stat_type: DescriptiveStatType::Count,
            })
        }

        if failures.is_empty() {
            checked_stats_record.passed = true;
            Ok(checked_stats_record)
        } else {
            Err((checked_stats_record, failures))
        }
    }
}

#[derive(Shrinkwrap)]
pub struct StatCheckResult(
    HashMap<GroupingKey, Result<CheckedStatsRecord, (CheckedStatsRecord, Vec<StatFailure>)>>,
);

impl Display for StatCheckResult {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for (stat_name, stat_result) in self.iter() {
            match stat_result {
                Ok(_checked_stat) => {
                    write!(f, "Checking {} metric... ok!\n", stat_name)?;
                }
                Err((_checked_stat, stat_failures)) => {
                    write!(f, "Checking {} metric... failed!\n", stat_name)?;
                    for stat_failure in stat_failures {
                        write!(f, "\t{}\n", stat_failure)?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl Commute for OnlineStats {
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

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
/// (metric name, run name)
pub struct GroupingKey(String, String);

impl Display for GroupingKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.0, self.1)
    }
}

impl GroupingKey {
    pub fn new<S1: Into<String>, S2: Into<String>>(stream_id: S1, metric: S2) -> Self {
        let stream_id = stream_id.into();
        let metric = metric.into();

        Self(stream_id, metric)
    }
}

/// All combined descriptive statistics mapped by name of the metric
#[derive(Shrinkwrap, Debug, Clone)]
#[shrinkwrap(mutable)]
pub struct StatsByMetric<D: DescriptiveStats>(pub HashMap<GroupingKey, D>);

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_METRICS)]
impl<'a, D: DescriptiveStats + Clone + 'a> StatsByMetric<D> {
    pub fn to_records(&self) -> Box<dyn Iterator<Item = StatsRecord> + 'a> {
        let me = self.0.clone();
        Box::new(
            me.into_iter()
                .map(|(key, stat)| StatsRecord::new(key.0, key.1, stat)),
        )
    }

    pub fn write_csv<W: std::io::Write>(&self, write: W) -> Result<(), Box<dyn std::error::Error>> {
        let mut writer = csv::Writer::from_writer(write);
        let records = self.to_records();
        for record in records {
            writer.serialize(record)?;
        }
        writer.flush()?;
        Ok(())
    }

    /// Produces a hash map of descriptive statistics from `read` keyed by metric name.
    pub fn from_reader(
        read: &mut dyn std::io::Read,
    ) -> Result<StatsByMetric<StatsRecord>, Box<dyn Error>> {
        let mut reader = csv::Reader::from_reader(read);

        let mut data = HashMap::new();
        for record in reader.deserialize() {
            let stat: StatsRecord = record?;
            let metric_name = stat.metric.clone().map(|x| Ok(x)).unwrap_or_else(|| {
                Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "No metric name in stat record",
                )))
            })?;
            let stream_id = stat.stream_id.clone().map(|x| Ok(x)).unwrap_or_else(|| {
                Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "No stream id stat record",
                )))
            })?;
            data.insert(GroupingKey::new(stream_id, metric_name), stat);
        }

        Ok(StatsByMetric(data))
    }
}

impl<D: DescriptiveStats> StatsByMetric<D> {
    pub fn empty() -> StatsByMetric<D> {
        Self(HashMap::new())
    }
}

impl<D: DescriptiveStats> Default for StatsByMetric<D> {
    fn default() -> Self {
        Self::empty()
    }
}

impl std::iter::FromIterator<Metric> for StatsByMetric<OnlineStats> {
    fn from_iter<I: IntoIterator<Item = Metric>>(source: I) -> StatsByMetric<OnlineStats> {
        StatsByMetric(source.into_iter().fold(
            HashMap::new(),
            |mut stats_by_metric_name, metric| {
                let entry = stats_by_metric_name.entry(GroupingKey::new(
                    metric.stream_id.unwrap_or_else(String::new),
                    metric.name,
                ));

                let online_stats = entry.or_insert_with(OnlineStats::empty);
                online_stats.add(metric.value);
                stats_by_metric_name
            },
        ))
    }
}

impl StatsByMetric<OnlineStats> {
    pub fn group_by_regex<I: IntoIterator<Item = Metric>>(
        re: &Regex,
        metrics: I,
    ) -> StatsByMetric<OnlineStats> {
        StatsByMetric(metrics.into_iter().fold(HashMap::new(), |mut map, metric| {
            let metric_name = metric.name.clone();
            let stream_id = metric.stream_id.clone();
            stream_id
                .and_then(|stream_id| {
                    re.captures_iter(stream_id.as_str()).next().map(|captured| {
                        let key = GroupingKey::new(captured[1].to_string(), metric_name);
                        let entry = map.entry(key);
                        let stats: &mut OnlineStats = entry.or_insert_with(OnlineStats::empty);
                        stats.add(metric.value)
                    })
                })
                .unwrap_or_else(|| {});
            map
        }))
    }
}

impl<D: DescriptiveStats> FromIterator<(GroupingKey, D)> for StatsByMetric<D> {
    fn from_iter<I: IntoIterator<Item = (GroupingKey, D)>>(source: I) -> StatsByMetric<D> {
        StatsByMetric(source.into_iter().fold(
            HashMap::new(),
            |mut stats_by_metric_name, (s, d)| {
                stats_by_metric_name.insert(s, d);
                stats_by_metric_name
            },
        ))
    }
}

impl Commute for StatsByMetric<OnlineStats> {
    fn merge(&mut self, rhs: Self) {
        for (key, online_stats_rhs) in rhs.iter() {
            let entry = self.entry(key.clone());
            let online_stats = entry.or_insert_with(OnlineStats::empty);
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
            .map(|x| Metric::new("latency", Some("test".into()), None, x));
        let size_data = vec![1.0, 10.0, 100.0]
            .into_iter()
            .map(|x| Metric::new("size", Some("test".into()), None, x));
        let all_data = latency_data.chain(size_data);
        let stats = StatsByMetric::from_iter(all_data);

        let latency_stats = stats
            .get(&GroupingKey::new("test", "latency"))
            .expect("latency stats to be present");

        assert_eq!(latency_stats.mean(), 100.0);
        let size_stats = stats
            .get(&GroupingKey::new("test", "size"))
            .expect("size stats to be present");

        assert_eq!(size_stats.mean(), 37.0);

        assert_eq!(latency_stats.min(), 50.0);
        assert_eq!(latency_stats.max(), 150.0);

        assert_eq!(size_stats.min(), 1.0);
        assert_eq!(size_stats.max(), 100.0);
    }

    #[test]
    fn can_perform_stat_check() {
        let expected: StatsByMetric<StatsRecord> = StatsByMetric::from_iter(
            vec![(
                GroupingKey::new("test", "latency"),
                StatsRecord {
                    mean: 50.0,
                    max: 100.0,
                    min: 25.0,
                    cnt: 100.,
                    stddev: 10.0,
                    variance: 5.0,
                    ..Default::default()
                },
            )]
            .into_iter(),
        );

        let actual: StatsByMetric<StatsRecord> = StatsByMetric::from_iter(
            vec![(
                GroupingKey::new("test", "latency"),
                StatsRecord {
                    mean: 75.0,
                    max: 150.0,
                    min: 50.0,
                    cnt: 100.0,
                    stddev: 20.0,
                    variance: 8.0,
                    ..Default::default()
                },
            )]
            .into_iter(),
        );

        let actual = format!(
            "{}",
            LessThanStatCheck::default().check_all(&expected, &actual)
        );
        let expected = "Checking test: latency metric... failed!\n\tMean: Expected 50, Actual was 75\n\tStdDev: Expected 10, Actual was 20\n\tMax: Expected 100, Actual was 150\n\tMin: Expected 25, Actual was 50\n";

        assert_eq!(expected, actual);
    }

    #[test]
    fn percent_change_works() {
        assert_eq!(0.50, percent_change(10.0, 15.0));
        assert_eq!(1. / 3., percent_change(15.0, 10.0));
        assert!(f64::is_infinite(percent_change(0., 10.0)));
    }

    #[test]
    fn checked_stats_can_serialize() {
        let expected = StatsRecord {
            mean: 50.0,
            max: 100.0,
            min: 25.0,
            cnt: 100.0,
            stddev: 10.0,
            variance: 5.0,
            ..Default::default()
        };
        let actual = StatsRecord {
            mean: 60.0,
            max: 150.0,
            min: 35.0,
            cnt: 100.0,
            stddev: 15.0,
            variance: 8.0,
            ..Default::default()
        };

        let percent_change_allowed = 0.05;
        let checked = CheckedStatsRecord::new(&expected, &actual, percent_change_allowed, false);

        let mut writer = csv::Writer::from_writer(std::io::stdout());
        writer.serialize(checked).unwrap();
        writer.flush().unwrap();
    }
}
