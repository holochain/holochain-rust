use crate::*;

use rusoto_cloudwatch::{CloudWatch, CloudWatchClient, MetricDatum, PutMetricDataInput};
use rusoto_core::region::Region;
use rusoto_logs::*;
use std::{
    convert::{TryFrom, TryInto},
    iter::FromIterator,
    time::UNIX_EPOCH,
};

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
    type Error = std::num::ParseFloatError;
    fn try_from(result_field: ResultField) -> Result<Self, Self::Error> {
        let metric_name = result_field
            .field
            .unwrap_or_else(|| "unlabeled".to_string());
        let metric_value: f64 = result_field.value.unwrap_or_default().parse()?;
        Ok(Metric::new(&metric_name, metric_value))
    }
}

impl TryFrom<&ResultField> for Metric {
    type Error = std::num::ParseFloatError;
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

#[derive(Clone)]
pub struct CloudWatchLogger {
    pub client: CloudWatchLogsClient,
    pub log_group_name: Option<String>,
    pub log_stream_name: Option<String>,
}

const LOG_LIMIT: i64 = 1000000;
impl CloudWatchLogger {
    pub fn query(
        &self,
        start_time: &std::time::SystemTime,
        end_time: &std::time::SystemTime,
    ) -> Vec<Vec<ResultField>> {
        let query_string = "@message like /metrics.rs/".to_string();
        let start_query_request = StartQueryRequest {
            limit: Some(LOG_LIMIT),
            query_string,
            start_time: start_time.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
            end_time: end_time.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
            log_group_name: self.log_group_name.clone(),
            log_group_names: None,
        };

        let query: StartQueryResponse =
            self.client.start_query(start_query_request).sync().unwrap();

        let get_query_results_request = GetQueryResultsRequest {
            query_id: query.query_id.unwrap(),
        };

        let query_result: GetQueryResultsResponse = self
            .client
            .get_query_results(get_query_results_request)
            .sync()
            .unwrap();

        let log_records = query_result.results.unwrap_or_default();

        log_records
    }

    pub fn query_metrics(
        &self,
        start_time: &std::time::SystemTime,
        end_time: &std::time::SystemTime,
    ) -> Box<dyn Iterator<Item = Metric>> {
        let query = self.query(start_time, end_time);
        let iterator = query
            .into_iter()
            .map(|result_vec| {
                result_vec.into_iter().map(|result_field| {
                    let metric: Metric = result_field.try_into().unwrap();
                    metric
                })
            })
            .flatten();

        Box::new(iterator)
    }

    pub fn query_and_aggregate(
        &self,
        start_time: &std::time::SystemTime,
        end_time: &std::time::SystemTime,
    ) -> crate::stats::Stats {
        crate::stats::Stats::from_iter(self.query_metrics(start_time, end_time))
    }

    pub fn default_log_stream() -> String {
        format!("holochain-{:?}", snowflake::ProcessUniqueId::new())
    }

    pub fn default_log_group() -> String {
        "holochain".to_string()
    }

    pub fn with_log_stream(
        log_stream_name: String,
        log_group_name: String,
        region: &Region,
    ) -> Self {
        let client = CloudWatchLogsClient::new(region.clone());

        let log_stream_request = CreateLogStreamRequest {
            log_group_name: log_group_name.clone(),
            log_stream_name: log_stream_name.clone(),
        };

        client.create_log_stream(log_stream_request).sync().unwrap();
        Self {
            client,
            log_stream_name: Some(log_stream_name),
            log_group_name: Some(log_group_name),
        }
    }
}

impl MetricPublisher for CloudWatchLogger {
    fn publish(&mut self, metric: &Metric) {
        let input_log_event = InputLogEvent {
            message: format!("metrics.rs: {} {}", metric.name, metric.value),
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
            sequence_token: None,
        };
        self.client.put_log_events(put_log_events_request);
    }
}

impl Default for CloudWatchLogger {
    fn default() -> Self {
        let default_log_stream = Self::default_log_stream();
        let default_log_group = Self::default_log_group();
        CloudWatchLogger::with_log_stream(default_log_stream, default_log_group, &DEFAULT_REGION)
    }
}
