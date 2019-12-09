/// Cloudwatch support for metric publising, aggregating, and analysis
use crate::*;

use crate::{
    logger::{LogLine, ParseError},
    stats::{GroupingKey, OnlineStats, StatsByMetric},
};
use rusoto_core::region::Region;
use rusoto_logs::*;
use std::{
    convert::{TryFrom, TryInto},
    time::{SystemTime, UNIX_EPOCH},
};

use std::collections::{HashMap, HashSet};
use structopt::StructOpt;

use rusoto_sts::{StsAssumeRoleSessionCredentialsProvider, StsClient};

pub const DEFAULT_REGION: Region = Region::EuCentral1;

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
    pub metrics_to_publish: Vec<Metric>,
}

impl Drop for CloudWatchLogger {
    fn drop(&mut self) {
        self.publish_internal()
    }
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
    ) -> Box<dyn Iterator<Item = (String, Metric)> + 'a> {
        let iterator = query.into_iter().filter_map(|result_vec| {
            let (log_stream_name, metric) = result_vec.into_iter().fold(
                (None, None),
                |(log_stream_name, metric), result_field| {
                    let field = result_field.clone().field.unwrap_or_default();
                    if field == "@message" {
                        let metric: Metric = result_field.try_into().unwrap();
                        (log_stream_name, Some(metric))
                    } else if field == "@logStream" {
                        (result_field.value, metric)
                    } else {
                        (log_stream_name, metric)
                    }
                },
            );

            log_stream_name
                .and_then(|log_stream_name| metric.map(|metric| (log_stream_name, metric)))
        });
        Box::new(iterator)
    }

    /// Queries cloudwatch logs given a start and end time interval and produces
    /// all metric samples observed during the interval
    pub fn query_metrics(
        &self,
        query_args: &QueryArgs,
    ) -> Box<dyn Iterator<Item = (String, Metric)>> {
        let query = self.query(query_args);
        Self::metrics_of_query(query)
    }

    /// Queries cloudwatch logs given a start and end time interval and produces
    /// aggregate statistics of metrics from the results.
    pub fn query_and_aggregate(&self, query_args: &QueryArgs) -> StatsByMetric<OnlineStats> {
        let mut hash_map = HashMap::new();

        for (log_stream, metric) in self.query_metrics(query_args) {
            let scenario_data_result: Result<ScenarioData, _> = log_stream.clone().try_into();

            if let Err(e) = scenario_data_result {
                debug!("Not a valid log stream pattern, skipping: {:?}", e);
                continue;
            }

            let entry = hash_map.entry(GroupingKey::new(log_stream, metric.name));

            let stats = entry.or_insert_with(OnlineStats::empty);
            stats.add(metric.value);
        }
        StatsByMetric(hash_map)
    }

    pub fn default_log_stream() -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        format!("holochain-{}", now)
    }

    pub fn default_log_group() -> String {
        "/aws/ec2/holochain/performance/".to_string()
    }

    pub fn default_start_time() -> i64 {
        UNIX_EPOCH.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
    }

    pub fn default_assume_role_arn() -> String {
        FINAL_EXAM_NODE_ROLE.to_string()
    }

    pub fn default_end_time() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    pub fn with_log_group<P: rusoto_credential::ProvideAwsCredentials + Sync + Send + 'static>(
        log_group_name: String,
        credentials_provider: P,
        region: &Region,
    ) -> Self
    where
        P::Future: Send,
    {
        Self::new(None, Some(log_group_name), credentials_provider, region)
    }

    pub fn with_region(region: &Region) -> Self {
        let client = CloudWatchLogsClient::new(region.clone());
        Self {
            client,
            log_stream_name: None,
            log_group_name: None,
            sequence_token: None,
            metrics_to_publish: vec![],
        }
    }

    pub fn ensure_log_group(&self) {
        if let Some(log_group_name) = &self.log_group_name {
            let log_group_request = CreateLogGroupRequest {
                log_group_name: log_group_name.clone(),
                ..Default::default()
            };
            // TODO Check if log group already exists or set them up a priori
            self.client
                .create_log_group(log_group_request)
                .sync()
                .unwrap_or_else(|e| {
                    debug!("Could not create log group- maybe already created: {:?}", e)
                });
        }
    }

    pub fn new<P: rusoto_credential::ProvideAwsCredentials + Sync + Send + 'static>(
        log_stream_name: Option<String>,
        log_group_name: Option<String>,
        credentials_provider: P,
        region: &Region,
    ) -> Self
    where
        P::Future: Send,
    {
        let client = CloudWatchLogsClient::new_with(
            rusoto_core::request::HttpClient::new().unwrap(),
            credentials_provider,
            region.clone(),
        );

        let mut log_group_name = log_group_name;
        if let Some(log_stream_name) = &log_stream_name {
            let log_group_name2 = log_group_name.unwrap_or_default();
            let log_stream_request = CreateLogStreamRequest {
                log_group_name: log_group_name2.clone(),
                log_stream_name: log_stream_name.clone(),
            };

            log_group_name = Some(log_group_name2);
            // TODO check if log stream already exists
            client
                .create_log_stream(log_stream_request)
                .sync()
                .unwrap_or_else(|e| {
                    debug!(
                        "Failed to create log stream, maybe it's already created: {:?}",
                        e
                    )
                });
        }

        debug!(
            "cloudwatch logger instance created for log_stream {:?} and log_group {:?}",
            log_stream_name, log_group_name
        );

        Self {
            client,
            log_stream_name: log_stream_name,
            log_group_name: log_group_name,
            sequence_token: None,
            metrics_to_publish: vec![],
        }
    }
}

const PUBLISH_CHUNK_SIZE: usize = 100;

impl MetricPublisher for CloudWatchLogger {
    fn publish(&mut self, metric: &Metric) {
        self.metrics_to_publish.push(metric.clone());

        if self.metrics_to_publish.len() < PUBLISH_CHUNK_SIZE {
            return;
        }
        self.publish_internal();
    }
}

impl CloudWatchLogger {
    fn publish_internal(&mut self) {
        let log_events = self
            .metrics_to_publish
            .drain(..)
            .map(|metric| {
                let log_line: LogLine = metric.into();
                InputLogEvent {
                    message: format!("metrics.rs: {}", log_line.to_string()),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as i64,
                }
            })
            .collect::<Vec<InputLogEvent>>();

        let put_log_events_request = PutLogEventsRequest {
            log_events,
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

    pub fn get_log_stream_names<S: Into<String>>(
        &self,
        log_stream_name_prefix: S,
    ) -> Box<dyn Iterator<Item = String>> {
        let log_stream_name_prefix = Some(log_stream_name_prefix.into());

        let log_group_name = self
            .log_group_name
            .clone()
            .unwrap_or_else(CloudWatchLogger::default_log_group);
        let request = DescribeLogStreamsRequest {
            log_group_name,
            log_stream_name_prefix,
            ..Default::default()
        };

        let response = self
            .client
            .describe_log_streams(request)
            .sync()
            .unwrap_or_else(|e| panic!("Problem querying log streams: {:?}", e));

        response
            .log_streams
            .map(|log_streams| {
                Box::new(
                    log_streams
                        .into_iter()
                        .filter_map(|log_stream| log_stream.log_stream_name),
                ) as Box<dyn Iterator<Item = String>>
            })
            .unwrap_or_else(|| Box::new(vec![].into_iter()) as Box<dyn Iterator<Item = String>>)
    }
}

impl Default for CloudWatchLogger {
    fn default() -> Self {
        let default_log_stream = Self::default_log_stream();
        let default_log_group = Self::default_log_group();
        CloudWatchLogger::new(
            Some(default_log_stream),
            Some(default_log_group),
            rusoto_credential::InstanceMetadataProvider::new(),
            &DEFAULT_REGION,
        )
    }
}

const LOG_STREAM_SEPARATOR: &str = ".";

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ScenarioData {
    run_name: String,
    net_name: String,
    dna_name: String,
    scenario_name: String,
    player_name: String,
}

impl Into<String> for ScenarioData {
    fn into(self: Self) -> String {
        let s = self;
        format!(
            "{}.{}.{}.{}.{}",
            s.run_name, s.net_name, s.dna_name, s.scenario_name, s.player_name
        )
    }
}

impl TryFrom<String> for ScenarioData {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let split = s.split(LOG_STREAM_SEPARATOR).collect::<Vec<_>>();
        if split.len() < 4 {
            return Err(format!(
                "Log stream name doesn't have at least 4 path elements: {:?}",
                split
            ));
        }
        Ok(Self {
            run_name: split[0].into(),
            net_name: split[1].into(),
            dna_name: split[2].into(),
            scenario_name: split[3].into(),
            player_name: split[4].into(),
        })
    }
}

impl TryFrom<LogStream> for ScenarioData {
    type Error = String;
    fn try_from(log_stream: LogStream) -> Result<Self, Self::Error> {
        let result: Result<Self, Self::Error> = log_stream
            .log_stream_name
            .map(|x| Ok(x))
            .unwrap_or_else(|| Err(format!("Log stream name missing")))
            .and_then(TryFrom::try_from);
        result
    }
}
impl ScenarioData {
    /// Groups by everything _but_ the player name
    fn without_player_name(&self) -> String {
        format!(
            "{}.{}.{}.{}",
            self.run_name, self.net_name, self.dna_name, self.scenario_name
        )
    }
}

/// Groups a log stream name by its scenario name, which is by convention is the 2nd to last field.
/// Eg. "2019-12-06_01-54-47_stress_10_1_2.sim2h.smoke.9"
pub fn group_by_scenario(
    log_stream_names: &mut dyn Iterator<Item = String>,
) -> HashMap<String, HashSet<ScenarioData>> {
    log_stream_names.fold(HashMap::new(), |mut grouped, log_stream_name| {
        let scenario_data: Result<ScenarioData, _> = log_stream_name.try_into();
        if let Ok(scenario_data) = scenario_data {
            grouped
                .entry(scenario_data.without_player_name())
                .or_insert_with(HashSet::new)
                .insert(scenario_data);
        }
        grouped
    })
}

pub const FINAL_EXAM_NODE_ROLE: &str =
    "arn:aws:iam::024992937548:role/ecs-stress-test-lambda-role-eu-central-1";

pub fn assume_role(region: &Region, role_arn: &str) -> StsAssumeRoleSessionCredentialsProvider {
    let sts = StsClient::new_with(
        rusoto_core::request::HttpClient::new().unwrap(),
        rusoto_credential::InstanceMetadataProvider::new(),
        region.clone(),
    );

    let provider = StsAssumeRoleSessionCredentialsProvider::new(
        sts,
        role_arn.to_owned(),
        format!(
            "hc-metrics-{}",
            snowflake::ProcessUniqueId::new().to_string()
        ),
        None,
        None,
        None,
        None,
    );
    provider
}
