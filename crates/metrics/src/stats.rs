/// Provides statistical features over metric data.
use crate::Metric;
use num_traits::float::Float;
/// Extends the metric api with statistical aggregation functions
use stats::Commute;
use std::{
    collections::HashMap,
    error::Error,
    fmt,
    fmt::{Display, Formatter},
    io,
    iter::FromIterator,
};

pub trait DescriptiveStats {
    fn max(&self) -> f64;
    fn min(&self) -> f64;
    fn cnt(&self) -> f64;
    fn mean(&self) -> f64;
    fn stddev(&self) -> f64;
    fn variance(&self) -> f64;

    fn percent_diff(&self, other: &dyn DescriptiveStats) -> StatsRecord {
        StatsRecord {
            mean: percent_diff(self.mean(), other.mean()),
            max: percent_diff(self.max(), other.max()),
            min: percent_diff(self.min(), other.min()),
            cnt: percent_diff(self.cnt(), other.cnt()),
            stddev: percent_diff(self.stddev(), other.stddev()),
            variance: percent_diff(self.variance(), other.variance()),
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatsRecord {
    pub name: Option<String>,
    pub max: f64,
    pub min: f64,
    pub cnt: f64,
    pub mean: f64,
    pub variance: f64,
    pub stddev: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CheckedStatsRecord {
    scenario_name: String,
    metric_name: String,
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
    pub percent_diff_max: f64,
    pub percent_diff_min: f64,
    pub percent_diff_cnt: f64,
    pub percent_diff_mean: f64,
    pub percent_diff_variance: f64,
    pub percent_diff_stddev: f64,
    pub percent_diff_allowed: f64,
    pub passed: bool,
}

impl CheckedStatsRecord {
    pub fn new<S: Into<String>>(
        scenario_name: S,
        metric_name: S,
        expected: &dyn DescriptiveStats,
        actual: &dyn DescriptiveStats,
        percent_diff_allowed: f64,
    ) -> Self {
        let percent_diff = expected.percent_diff(actual);
        let scenario_name = scenario_name.into();
        let metric_name = metric_name.into();
        Self {
            scenario_name,
            metric_name,
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
            percent_diff_max: percent_diff.max(),
            percent_diff_min: percent_diff.min(),
            percent_diff_cnt: percent_diff.cnt(),
            percent_diff_mean: percent_diff.mean(),
            percent_diff_variance: percent_diff.variance(),
            percent_diff_stddev: percent_diff.stddev(),
            percent_diff_allowed,
            passed: LessThanStatCheck {
                percent_diff_allowed,
            }
            .check(expected, actual)
            .is_ok(),
        }
    }
}
impl Default for StatsRecord {
    fn default() -> Self {
        Self {
            name: None,
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
    pub fn new<S: Into<String>, D: DescriptiveStats>(metric_name: S, desc: D) -> Self {
        let metric_name = metric_name.into();
        Self {
            name: Some(metric_name),
            max: desc.max(),
            min: desc.min(),
            mean: desc.mean(),
            cnt: desc.cnt() as f64,
            stddev: desc.stddev(),
            variance: desc.variance(),
        }
    }

    pub fn from_reader(
        read: &mut dyn std::io::Read,
    ) -> Result<HashMap<String, Box<dyn DescriptiveStats>>, Box<dyn Error>> {
        let mut reader = csv::Reader::from_reader(read);

        let mut stats_by_metric_name: HashMap<String, Box<dyn DescriptiveStats>> = HashMap::new();
        for record in reader.deserialize() {
            let stat: StatsRecord = record?;
            let stat_name = stat.name.clone().map(|x| Ok(x)).unwrap_or_else(|| {
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
            name: None,
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
        expected: &dyn DescriptiveStats,
        actual: &dyn DescriptiveStats,
    ) -> Result<(), Vec<StatFailure>>;

    fn check_all(
        &self,
        expected: HashMap<String, Box<dyn DescriptiveStats>>,
        actual: HashMap<String, Box<dyn DescriptiveStats>>,
    ) -> StatCheckResult {
        StatCheckResult(HashMap::from_iter(expected.iter().map(
            |(stat_name, expected_stat)| {
                let result = if let Some(actual_stat) = actual.get(stat_name) {
                    self.check(expected_stat.as_ref(), actual_stat.as_ref())
                } else {
                    Err(vec![])
                };
                (stat_name.clone(), result)
            },
        )))
    }
}

/// Computes percentage different between expected and actual
pub fn percent_diff<N: Into<f64>>(expected: N, actual: N) -> f64 {
    let e = expected.into();
    let a = actual.into();
    f64::abs(e - a) / e
}

#[derive(Clone, Debug)]
pub struct LessThanStatCheck {
    percent_diff_allowed: f64,
}

impl LessThanStatCheck {}

impl Default for LessThanStatCheck {
    fn default() -> Self {
        LessThanStatCheck {
            percent_diff_allowed: 0.05,
        }
    }
}

impl StatCheck for LessThanStatCheck {
    fn check(
        &self,
        expected: &dyn DescriptiveStats,
        actual: &dyn DescriptiveStats,
    ) -> Result<(), Vec<StatFailure>> {
        let percent_diff = expected.percent_diff(actual);

        let mut failures = Vec::new();

        if percent_diff.mean() > self.percent_diff_allowed {
            failures.push(StatFailure {
                expected: expected.mean(),
                actual: actual.mean(),
                stat_type: DescriptiveStatType::Mean,
            })
        }

        if percent_diff.stddev() > self.percent_diff_allowed {
            failures.push(StatFailure {
                expected: expected.stddev(),
                actual: actual.stddev(),
                stat_type: DescriptiveStatType::StdDev,
            })
        }

        if percent_diff.max() > self.percent_diff_allowed {
            failures.push(StatFailure {
                expected: expected.max(),
                actual: actual.max(),
                stat_type: DescriptiveStatType::Max,
            })
        }

        if percent_diff.min() > self.percent_diff_allowed {
            failures.push(StatFailure {
                expected: expected.min(),
                actual: actual.min(),
                stat_type: DescriptiveStatType::Min,
            })
        }

        if percent_diff.cnt() > self.percent_diff_allowed {
            failures.push(StatFailure {
                expected: expected.cnt() as f64,
                actual: actual.cnt() as f64,
                stat_type: DescriptiveStatType::Count,
            })
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[derive(Shrinkwrap)]
pub struct StatCheckResult(HashMap<String, Result<(), Vec<StatFailure>>>);

impl Display for StatCheckResult {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for (stat_name, stat_result) in self.iter() {
            match stat_result {
                Ok(_) => {
                    write!(f, "Checking {} metric... ok!\n", stat_name)?;
                }
                Err(stat_failures) => {
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

/// All combined descriptive statistics mapped by name of the metric
#[derive(Shrinkwrap, Debug, Clone)]
pub struct StatsByMetric(pub HashMap<String, OnlineStats>);

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

                let online_stats = entry.or_insert_with(OnlineStats::empty);
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

    #[test]
    fn can_perform_stat_check() {
        let expected: HashMap<String, Box<dyn DescriptiveStats>> = HashMap::from_iter(
            vec![(
                "latency".to_string(),
                Box::new(StatsRecord {
                    mean: 50.0,
                    max: 100.0,
                    min: 25.0,
                    cnt: 100.,
                    stddev: 10.0,
                    variance: 5.0,
                    ..Default::default()
                }) as Box<dyn DescriptiveStats>,
            )]
            .into_iter(),
        );

        let actual: HashMap<String, Box<dyn DescriptiveStats>> = HashMap::from_iter(
            vec![(
                "latency".to_string(),
                Box::new(StatsRecord {
                    mean: 75.0,
                    max: 150.0,
                    min: 50.0,
                    cnt: 100.0,
                    stddev: 20.0,
                    variance: 8.0,
                    ..Default::default()
                }) as Box<dyn DescriptiveStats>,
            )]
            .into_iter(),
        );

        let actual = format!(
            "{}",
            LessThanStatCheck::default().check_all(expected, actual)
        );
        let expected = "Checking latency metric... failed!\n\tMean: Expected 50, Actual was 75\n\tStdDev: Expected 10, Actual was 20\n\tMax: Expected 100, Actual was 150\n\tMin: Expected 25, Actual was 50\n";

        assert_eq!(expected, actual);
    }

    #[test]
    fn percent_diff_works() {
        assert_eq!(0.50, percent_diff(10.0, 15.0));
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

        let percent_diff_allowed = 0.05;
        let checked = CheckedStatsRecord::new(
            "direct message",
            "zome_call.commit.latency",
            &expected,
            &actual,
            percent_diff_allowed,
        );

        let mut writer = csv::Writer::from_writer(std::io::stdout());
        writer.serialize(checked).unwrap();
        writer.flush().unwrap();
    }
}
