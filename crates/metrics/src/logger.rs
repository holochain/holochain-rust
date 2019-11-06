use crate::{Metric, MetricPublisher};
use regex::Regex;
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub struct LoggerMetricPublisher;

impl LoggerMetricPublisher {
    pub fn new() -> Self {
        Self
    }
}

impl MetricPublisher for LoggerMetricPublisher {
    fn publish(&mut self, metric: &Metric) {
        let log_line: LogLine = metric.into();
        debug!("{}", log_line.to_string());
    }
}

impl Default for LoggerMetricPublisher {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static! {
    pub static ref PARSE_METRIC_REGEX: Regex =
        Regex::new("metrics.rs:\\d* ([\\w\\d~\\-\\.]+) ([\\d\\.]+)").unwrap();
}

#[derive(Debug, Clone)]
pub struct ParseError(pub String);

impl From<std::num::ParseFloatError> for ParseError {
    fn from(f: std::num::ParseFloatError) -> Self {
        ParseError(format!("Couldn't convert metric value to f64: {:?}", f))
    }
}

#[derive(Debug, Clone, Shrinkwrap)]
pub struct LogLine(pub String);

impl From<Metric> for LogLine {
    fn from(metric: Metric) -> Self {
        LogLine(format!("{} {}", metric.name, metric.value))
    }
}

impl From<&Metric> for LogLine {
    fn from(metric: &Metric) -> Self {
        metric.clone().into()
    }
}

impl Into<String> for LogLine {
    fn into(self) -> String {
        self.0
    }
}

impl TryFrom<LogLine> for Metric {
    type Error = ParseError;
    fn try_from(source: LogLine) -> Result<Metric, ParseError> {
        let cap = PARSE_METRIC_REGEX
            .captures_iter(source.as_str())
            .next()
            .map(|cap| Ok(cap))
            .unwrap_or_else(|| {
                Err(ParseError(format!(
                    "expected at least one capture group for a metric value: {:?}",
                    source
                )))
            })?;
        let metric_name: String = cap[1].to_string();
        let value_str = cap[2].to_string();
        let metric_value: f64 = value_str.as_str().parse()?;
        let metric = Metric::new(&metric_name, metric_value);
        return Ok(metric);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::convert::TryInto;
    #[test]
    fn can_convert_log_line_to_metric() {
        let line =
            "DEBUG 2019-10-30 10:34:44 [holochain_metrics::metrics] net_worker_thread/puid-4-2e crates/metrics/src/metrics.rs:33 sim2h_worker.tick.latency 123";
        let log_line = LogLine(line.to_string());
        let metric: Metric = log_line.try_into().unwrap();
        assert_eq!("sim2h_worker.tick.latency", metric.name);
        assert_eq!(123.0, metric.value);
    }

    #[test]
    fn can_convert_cloudwatch_log_line_to_metric() {
        let line = "metrics.rs: sim2h_worker.tick.latency 123";
        let log_line = LogLine(line.to_string());
        let metric: Metric = log_line.try_into().unwrap();
        assert_eq!("sim2h_worker.tick.latency", metric.name);
        assert_eq!(123.0, metric.value);
    }

}
