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
        debug!("{} {}", metric.name, metric.value);
    }
}

lazy_static! {
    pub static ref PARSE_METRIC_REGEX: Regex =
        Regex::new("metrics.rs:\\d+ ([\\w\\d~-\\.]+) ([\\d\\.]+)").unwrap();
}

#[derive(Debug, Clone)]
pub struct ParseError(String);

impl From<std::num::ParseFloatError> for ParseError {
    fn from(f: std::num::ParseFloatError) -> Self {
        ParseError(format!("Couldn't convert metric value to f64: {:?}", f))
    }
}

#[derive(Debug, Clone, Shrinkwrap)]
pub struct LogLine(String);

impl TryFrom<LogLine> for Metric {
    type Error = ParseError;
    fn try_from(source: LogLine) -> Result<Metric, ParseError> {
        for cap in PARSE_METRIC_REGEX.captures_iter(source.as_str()) {
            let metric_name: String = cap[0].to_string();
            let metric_value: f64 = cap[1].parse()?;
            let metric = Metric::new(&metric_name, metric_value);
            return Ok(metric);
        }

        Err(ParseError(format!(
            "No metrics found in source: {:?}",
            source
        )))
    }
}
