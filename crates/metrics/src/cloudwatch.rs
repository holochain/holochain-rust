/// Cloudwatch support for metric publising, aggregating, and analysis
use crate::*;

use crate::{
    logger::{LogLine, ParseError},
    stats::StatsByMetric,
};
use rusoto_cloudwatch::{CloudWatch, CloudWatchClient, MetricDatum, PutMetricDataInput};
use rusoto_core::region::Region;
use rusoto_logs::*;
use std::{
    convert::{TryFrom, TryInto},
    iter::FromIterator,
    time::{SystemTime, UNIX_EPOCH},
};

use structopt::StructOpt;

const DEFAULT_REGION: Region = Region::EuCentral1;

#[derive(Clone)]
pub struct CloudWatchMetricPublisher {
    client: CloudWatchClient,
}

impl From<Metric> for MetricDatum {
    fn from(metric: Metric) -> Self {
        let cloud_watch_metric = MetricDatum {
            counts: None,
            dimensions: None,
            metric_name: metric.name.clone(),
            statistic_values: None,
            storage_resolution: None,
            timestamp: None,
            unit: None,
            value: Some(metric.value),
            values: None,
        };
        cloud_watch_metric
    }
}

impl From<&Metric> for MetricDatum {
    fn from(metric: &Metric) -> Self {
        let m: Self = metric.clone().into();
        m
    }
}

// TODO Test this
impl TryFrom<ResultField> for Metric {
    type Error = ParseError;
    fn try_from(result_field: ResultField) -> Result<Self, Self::Error> {
        if result_field.field != Some("@message".to_string()) {
            return Err(ParseError(format!(
                "Expected message field but got: {:?}",
                result_field.field
            )));
        }
        let message = result_field.value.map(|m| Ok(m)).unwrap_or_else(|| {
            Err(ParseError(
                "Expected message value but got none".to_string(),
            ))
        })?;
        let metric: Metric = LogLine(message).try_into()?;
        Ok(metric)
    }
}

impl TryFrom<&ResultField> for Metric {
    type Error = ParseError;
    fn try_from(result_field: &ResultField) -> Result<Self, Self::Error> {
        let r: Result<Self, Self::Error> = result_field.clone().try_into();
        r
    }
}

impl CloudWatchMetricPublisher {
    pub fn new(region: &Region) -> Self {
        let client = CloudWatchClient::new(region.clone());
        Self { client }
    }
}

impl Default for CloudWatchMetricPublisher {
    fn default() -> Self {
        CloudWatchMetricPublisher::new(&DEFAULT_REGION)
    }
}

impl MetricPublisher for CloudWatchMetricPublisher {
    fn publish(&mut self, metric: &Metric) {
        let cloud_watch_metric: MetricDatum = metric.into();
        let data_input = PutMetricDataInput {
            metric_data: vec![cloud_watch_metric],
            namespace: "".to_string(),
        };
        let _rusoto_future = self.client.put_metric_data(data_input);
        trace!("published metric to cloudwatch: {:?}", metric);
    }
}

/// A cloud watch logger instance with some
/// configuration and state as needed by
/// various service calls.
#[derive(Clone)]
pub struct CloudWatchLogger {
    /// The underlying cloudwatch logs client
    pub client: CloudWatchLogsClient,
    pub log_group_name: Option<String>,
    pub log_stream_name: Option<String>,
    /// Set automatically when publishing log metrics
    pub sequence_token: Option<String>,
}

#[derive(Clone, Debug, Default, StructOpt)]
pub struct QueryArgs {
    #[structopt(name = "start_time")]
    pub start_time: Option<i64>,
    #[structopt(name = "end_time")]
    pub end_time: Option<i64>,
    #[structopt(
        name = "log_stream_pat",
        short = "p",
        about = "The log stream pattern to filter messages over"
    )]
    pub log_stream_pat: Option<String>,
}

impl CloudWatchLogger {
    /// Query the cloudwatch logger given a start and stop time interval.
    /// Produces a raw vector of result field rows (each as a vector).
    /// Use `CloudWatchLogger::query_metrics` or `CloudWatchLogger::query_and_aggregate`
    /// to produce numerical data from this raw data.
    pub fn query(&self, query_args: &QueryArgs) -> Vec<Vec<ResultField>> {
        let query_string;

        if let Some(log_stream_pat) = &query_args.log_stream_pat {
            query_string =
                format!(
                "fields @message, @logStream | filter @message like '{}' and @logStream like '{}'",
                logger::METRIC_TAG, log_stream_pat);
        } else {
            query_string = format!(
                "fields @message | filter @message like '{}'",
                logger::METRIC_TAG
            );
        }

        let start_query_request = StartQueryRequest {
            limit: None,
            query_string,
            start_time: query_args
                .start_time
                .unwrap_or_else(Self::default_start_time),
            end_time: query_args.end_time.unwrap_or_else(Self::default_end_time),
            log_group_name: self
                .log_group_name
                .clone()
                .unwrap_or_else(Self::default_log_group), // This is optional for rusoto > 0.41.0
                                                          // log_group_names: None, <-- Uncomment for rusoto >= 0.41.0
        };

        let query: StartQueryResponse =
            self.client.start_query(start_query_request).sync().unwrap();

        let get_query_results_request = GetQueryResultsRequest {
            query_id: query.query_id.unwrap(),
        };

        let mut query_result: GetQueryResultsResponse;

        loop {
            query_result = self
                .client
                .get_query_results(get_query_results_request.clone())
                .sync()
                .unwrap();
            if query_result
                .status
                .map(|x| x == "Running")
                .unwrap_or_else(|| false)
            {
                std::thread::sleep(std::time::Duration::from_millis(20));
                continue;
            } else {
                break;
            }
        }

        let log_records = query_result.results.unwrap_or_default();

        log_records
    }

    /// Converts raw result fields to in iterator over metric samples
    pub fn metrics_of_query<'a>(
        query: Vec<Vec<ResultField>>,
    ) -> Box<dyn Iterator<Item = Metric> + 'a> {
        let iterator = query
            .into_iter()
            .map(|result_vec| {
                result_vec.into_iter().filter_map(|result_field| {
                    result_field.clone().field.and_then(|field| {
                        if field == "@message" {
                            let metric: Metric = result_field.try_into().unwrap();
                            Some(metric)
                        } else {
                            None
                        }
                    })
                })
            })
            .flatten();
        Box::new(iterator)
    }

    /// Queries cloudwatch logs given a start and end time interval and produces
    /// all metric samples observed during the interval
    pub fn query_metrics(&self, query_args: &QueryArgs) -> Box<dyn Iterator<Item = Metric>> {
        let query = self.query(query_args);
        Self::metrics_of_query(query)
    }

    /// Queries cloudwatch logs given a start and end time interval and produces
    /// aggregate statistics of metrics from the results.
    pub fn query_and_aggregate(&self, query_args: &QueryArgs) -> StatsByMetric {
        StatsByMetric::from_iter(self.query_metrics(query_args))
    }

    pub fn default_log_stream() -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        format!("holochain-{}", now)
    }

    pub fn default_log_group() -> String {
        "holochain".to_string()
    }

    pub fn default_start_time() -> i64 {
        UNIX_EPOCH.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
    }

    pub fn default_end_time() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    pub fn with_log_group(log_group_name: String, region: &Region) -> Self {
        let client = CloudWatchLogsClient::new(region.clone());

        let log_group_request = CreateLogGroupRequest {
            log_group_name: log_group_name.clone(),
            ..Default::default()
        };

        // TODO Check if log group already exists or set them up a priori
        client
            .create_log_group(log_group_request)
            .sync()
            .unwrap_or_else(|e| {
                debug!("Could not create log group- maybe already created: {:?}", e)
            });

        Self {
            client,
            log_stream_name: None,
            log_group_name: Some(log_group_name),
            sequence_token: None,
        }
    }

    pub fn with_region(region: &Region) -> Self {
        let client = CloudWatchLogsClient::new(region.clone());
        Self {
            client,
            log_stream_name: None,
            log_group_name: None,
            sequence_token: None,
        }
    }

    pub fn with_log_stream(
        log_stream_name: String,
        log_group_name: String,
        region: &Region,
    ) -> Self {
        let client = CloudWatchLogsClient::new(region.clone());

        let log_group_request = CreateLogGroupRequest {
            log_group_name: log_group_name.clone(),
            ..Default::default()
        };

        // TODO Check if log group already exists or set them up a priori
        client
            .create_log_group(log_group_request)
            .sync()
            .unwrap_or_else(|e| {
                debug!("Could not create log group- maybe already created: {:?}", e)
            });

        let log_stream_request = CreateLogStreamRequest {
            log_group_name: log_group_name.clone(),
            log_stream_name: log_stream_name.clone(),
        };

        client.create_log_stream(log_stream_request).sync().unwrap();
        Self {
            client,
            log_stream_name: Some(log_stream_name),
            log_group_name: Some(log_group_name),
            sequence_token: None,
        }
    }
}

impl MetricPublisher for CloudWatchLogger {
    fn publish(&mut self, metric: &Metric) {
        let log_line: LogLine = metric.into();
        let input_log_event = InputLogEvent {
            message: format!("metrics.rs: {}", log_line.to_string()),
            timestamp: std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
        };
        let put_log_events_request = PutLogEventsRequest {
            log_events: vec![input_log_event],
            log_group_name: self
                .log_group_name
                .clone()
                .unwrap_or_else(|| panic!("log_group_name must be set")),
            log_stream_name: self
                .log_stream_name
                .clone()
                .unwrap_or_else(|| panic!("log_stream_name must be set")),
            sequence_token: self.sequence_token.clone(),
        };
        let result = self
            .client
            .put_log_events(put_log_events_request)
            .sync()
            .unwrap();
        self.sequence_token = result.next_sequence_token
    }
}

impl Default for CloudWatchLogger {
    fn default() -> Self {
        let default_log_stream = Self::default_log_stream();
        let default_log_group = Self::default_log_group();
        CloudWatchLogger::with_log_stream(default_log_stream, default_log_group, &DEFAULT_REGION)
    }
}
