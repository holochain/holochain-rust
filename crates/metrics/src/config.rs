use crate::{cloudwatch::CloudWatchLogger, logger::LoggerMetricPublisher, MetricPublisher};
use holochain_locksmith::RwLock;
//use holochain_tracing_macros::newrelic_autotrace;
use std::sync::Arc;

/// Unifies all possible metric publisher configurations
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum MetricPublisherConfig {
    Logger,
    CloudWatchLogs(CloudWatchLogsConfig),
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct CloudWatchLogsConfig {
    #[serde(default)]
    pub region: Option<rusoto_core::region::Region>,
    #[serde(default)]
    pub log_group_name: Option<String>,
    #[serde(default)]
    pub log_stream_name: Option<String>,
    #[serde(default)]
    pub assume_role_arn: Option<String>,
}

impl Default for MetricPublisherConfig {
    fn default() -> Self {
        Self::default_logger()
    }
}

//#[newrelic_autotrace(HOLOCHAIN_METRICS)]
impl MetricPublisherConfig {
    /// Instantiates a new metric publisher given this configuration instance.
    pub fn create_metric_publisher(&self) -> Arc<RwLock<dyn MetricPublisher>> {
        let publisher: Arc<RwLock<dyn MetricPublisher>> = match self {
            Self::Logger => Arc::new(RwLock::new(LoggerMetricPublisher::new())),
            Self::CloudWatchLogs(CloudWatchLogsConfig {
                region,
                log_group_name,
                log_stream_name,
                assume_role_arn,
            }) => {
                let region = region.clone().unwrap_or_default();
                match &assume_role_arn {
                    Some(assume_role_arn) => Arc::new(RwLock::new(CloudWatchLogger::new(
                        log_stream_name.clone(),
                        log_group_name.clone(),
                        crate::cloudwatch::assume_role(&region, &assume_role_arn),
                        &region,
                    ))),

                    None => Arc::new(RwLock::new(CloudWatchLogger::new(
                        log_stream_name.clone(),
                        log_group_name.clone(),
                        rusoto_credential::InstanceMetadataProvider::new(),
                        &region,
                    ))),
                }
            }
        };
        publisher
    }

    /// The default logger metric publisher configuration.
    pub fn default_logger() -> Self {
        Self::Logger
    }

    /// The default cloudwatch logger publisher configuration.
    pub fn default_cloudwatch_logs() -> Self {
        Self::CloudWatchLogs(CloudWatchLogsConfig {
            region: Some(crate::cloudwatch::DEFAULT_REGION),
            log_group_name: Some(crate::cloudwatch::CloudWatchLogger::default_log_group()),
            log_stream_name: Some(crate::cloudwatch::CloudWatchLogger::default_log_stream()),
            assume_role_arn: None,
        })
    }
}
