use crate::{Metric, MetricPublisher};
use chrono::prelude::*;
use regex::Regex;
use std::{
    convert::{TryFrom, TryInto},
    io::{BufRead, BufReader},
    path::PathBuf,
};

/// A metric publisher that just logs to the debug level logger
/// a key value pair formatted according to the Into<String> trait of LogLine.
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
        debug!("{}", log_line);
    }
}

impl Default for LoggerMetricPublisher {
    fn default() -> Self {
        Self::new()
    }
}

pub const METRIC_TAG: &str = "METRIC";

lazy_static! {
    pub static ref LOG_HEADER_REGEX: Regex = Regex::new("[\\w]+ ([\\d\\-]+ [\\d:]+)").unwrap();
    pub static ref PARSE_METRIC_REGEX: Regex =
        Regex::new((METRIC_TAG.to_string() + " ([\\w\\d~\\-\\._]+) ([\\d\\.]+)").as_str()).unwrap();
}

#[derive(Debug, Clone)]
pub struct ParseError(pub String);

impl ParseError {
    pub fn new<S: Into<String>>(s: S) -> Self {
        Self(s.into())
    }
}

impl From<std::num::ParseFloatError> for ParseError {
    fn from(f: std::num::ParseFloatError) -> Self {
        ParseError(format!("Couldn't convert metric value to f64: {:?}", f))
    }
}

/// A metric as represented by a log line. Used to convert
/// from a metric to log line and back.
#[derive(Debug, Clone, Shrinkwrap)]
pub struct LogLine(pub String);

const RUST_LOG_DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

impl From<Metric> for LogLine {
    fn from(metric: Metric) -> Self {
        LogLine(format!(
            "{} {} {} {} {}",
            metric
                .timestamp
                .map(|t| t.format(RUST_LOG_DATE_FORMAT).to_string())
                .unwrap_or_else(String::new),
            metric.stream_id.unwrap_or_else(String::new),
            METRIC_TAG,
            metric.name,
            metric.value
        ))
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

impl std::fmt::Display for LogLine {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<LogLine> for Metric {
    type Error = ParseError;
    fn try_from(source: LogLine) -> Result<Metric, ParseError> {
        let stripped = strip_ansi_escapes::strip(source.0).unwrap();
        let stripped = std::str::from_utf8(stripped.as_slice()).unwrap();
        let cap = PARSE_METRIC_REGEX
            .captures_iter(stripped)
            .next()
            .map(|cap| Ok(cap))
            .unwrap_or_else(|| {
                Err(ParseError(format!(
                    "expected at least one capture group for a metric value: {:?}",
                    stripped
                )))
            })?;
        let timestamp = LOG_HEADER_REGEX
            .captures_iter(stripped)
            .next()
            .and_then(|s| {
                let to_parse = &s[1];
                NaiveDateTime::parse_from_str(to_parse, RUST_LOG_DATE_FORMAT)
                    .map_err(|e| warn!("Invalid date string {:?} from log: {:?}", to_parse, e))
                    .ok()
            })
            .map(|t: NaiveDateTime| {
                let dt: DateTime<_> = DateTime::from_utc(t, Utc);
                dt
            });
        let metric_name: String = cap[1].to_string();
        let value_str = cap[2].to_string();
        let metric_value: f64 = value_str.as_str().parse()?;
        let metric = Metric::new(&metric_name, None, timestamp, metric_value);
        return Ok(metric);
    }
}

/// Produces an iterator of metric data given a log file name.
pub fn metrics_from_file(log_file: PathBuf) -> std::io::Result<Box<dyn Iterator<Item = Metric>>> {
    let file = std::fs::File::open(log_file.clone())?;
    let reader = BufReader::new(file);
    let log_file: String = log_file.to_str().unwrap_or_else(|| "unknown").to_string();
    let stream_id = Some(log_file);

    let metrics = reader.lines().filter_map(move |line| {
        let result: Result<Metric, _> = line
            .map_err(|e| ParseError(format!("{}", e)))
            .and_then(move |line| LogLine(line).try_into());
        result
            .map(|mut metric| {
                metric.stream_id = stream_id.clone();
                Some(metric)
            })
            .unwrap_or_else(|e| {
                warn!("Unparsable log line: {:?}", e);
                None
            })
    });
    Ok(Box::new(metrics))
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::convert::TryInto;
    #[test]
    fn can_convert_log_line_to_metric() {
        let line = format!(
            "DEBUG 2019-10-30 10:34:44 [holochain_metrics::metrics] net_worker_thread/puid-4-2e crates/metrics/src/logger.rs:33 {} sim2h_worker.tick.latency 123", METRIC_TAG);
        let log_line = LogLine(line.to_string());
        let metric: Result<Metric, _> = log_line.try_into();

        assert!(metric.is_ok());

        let metric = metric.unwrap();
        assert_eq!("sim2h_worker.tick.latency", metric.name);
        assert_eq!(123.0, metric.value);
        assert_eq!(
            "2019-10-30 10:34:44",
            metric
                .timestamp
                .map(|x| x.format(RUST_LOG_DATE_FORMAT).to_string())
                .unwrap_or_else(|| "None".into())
        );
    }

    #[test]
    fn can_convert_cloudwatch_log_line_to_metric() {
        let line = format!("{} sim2h_worker.tick.latency 123", METRIC_TAG);
        let log_line = LogLine(line.to_string());
        let metric: Metric = log_line.try_into().unwrap();
        assert_eq!("sim2h_worker.tick.latency", metric.name);
        assert_eq!(123.0, metric.value);
    }
}
