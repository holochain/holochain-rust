use crate::*;

use rusoto_cloudwatch::{CloudWatch, CloudWatchClient, MetricDatum, PutMetricDataInput};
use rusoto_core::region::Region;
use rusoto_logs::*;
use std::time::UNIX_EPOCH;

const DEFAULT_REGION: Region = Region::UsEast1;

#[derive(Clone)]
pub struct CloudWatchMetricPublisher {
    client: CloudWatchClient,
}

impl From<Metric> for MetricDatum {
    fn from(metric: Metric) -> Self {
        let cloud_watch_metric = MetricDatum {
            /// <p>Array of numbers that is used along with the <code>Values</code> array. Each number in the <code>Count</code> array is the number of times the corresponding value in the <code>Values</code> array occurred during the period. </p> <p>If you omit the <code>Counts</code> array, the default of 1 is used as the value for each count. If you include a <code>Counts</code> array, it must include the same amount of values as the <code>Values</code> array.</p>
            counts: None,
            /// <p>The dimensions associated with the metric.</p>
            dimensions: None,
            /// <p>The name of the metric.</p>
            metric_name: metric.name.clone(),
            /// <p>The statistical values for the metric.</p>
            statistic_values: None,
            /// <p>Valid values are 1 and 60. Setting this to 1 specifies this metric as a high-resolution metric, so that CloudWatch stores the metric with sub-minute resolution down to one second. Setting this to 60 specifies this metric as a regular-resolution metric, which CloudWatch stores at 1-minute resolution. Currently, high resolution is available only for custom metrics. For more information about high-resolution metrics, see <a href="https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/publishingMetrics.html#high-resolution-metrics">High-Resolution Metrics</a> in the <i>Amazon CloudWatch User Guide</i>. </p> <p>This field is optional, if you do not specify it the default of 60 is used.</p>
            storage_resolution: None,
            /// <p>The time the metric data was received, expressed as the number of milliseconds since Jan 1, 1970 00:00:00 UTC.</p>
            timestamp: None,
            /// <p>When you are using a <code>Put</code> operation, this defines what unit you want to use when storing the metric.</p> <p>In a <code>Get</code> operation, this displays the unit that is used for the metric.</p>
            unit: None,
            /// <p>The value for the metric.</p> <p>Although the parameter accepts numbers of type Double, CloudWatch rejects values that are either too small or too large. Values must be in the range of 8.515920e-109 to 1.174271e+108 (Base 10) or 2e-360 to 2e360 (Base 2). In addition, special values (for example, NaN, +Infinity, -Infinity) are not supported.</p>
            value: Some(metric.value),
            /// <p>Array of numbers representing the values for the metric during the period. Each unique value is listed just once in this array, and the corresponding number in the <code>Counts</code> array specifies the number of times that value occurred during the period. You can include up to 150 unique values in each <code>PutMetricData</code> action that specifies a <code>Values</code> array.</p> <p>Although the <code>Values</code> array accepts numbers of type <code>Double</code>, CloudWatch rejects values that are either too small or too large. Values must be in the range of 8.515920e-109 to 1.174271e+108 (Base 10) or 2e-360 to 2e360 (Base 2). In addition, special values (for example, NaN, +Infinity, -Infinity) are not supported.</p>
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
    pub log_group_name: String,
}

const LOG_LIMIT: i64 = 1000000;
impl CloudWatchLogger {
    pub fn query(
        &self,
        start_time: &std::time::SystemTime,
        end_time: &std::time::SystemTime,
    ) -> Vec<Vec<ResultField>> {
        // TODO figure out what this should be
        let query_string = "".to_string();
        let start_query_request = StartQueryRequest {
            limit: Some(LOG_LIMIT),
            query_string,
            start_time: start_time.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
            end_time: end_time.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
            log_group_name: Some(self.log_group_name.clone()),
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
}

impl MetricPublisher for CloudWatchLogger {
    fn publish(&mut self, metric: &Metric) {
        let input_log_event = InputLogEvent {
            message: format!("{} {}", metric.name, metric.value),
            timestamp: std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
        };
        // TODO study stream api
        let log_stream_name = "TODO".to_string();
        let put_log_events_request = PutLogEventsRequest {
            log_events: vec![input_log_event],
            log_group_name: self.log_group_name.clone(),
            log_stream_name,
            sequence_token: None,
        };
        self.client.put_log_events(put_log_events_request);
    }
}
