use crate::{
    cloudwatch::{CloudWatchLogger, CloudWatchMetricPublisher},
    logger::LoggerMetricPublisher,
    MetricPublisher,
};
use holochain_locksmith::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MetricPublisherConfig {
    Logger,
    CloudWatchMetrics(Option<rusoto_core::region::Region>),
    CloudWatchLogs {
        region: Option<rusoto_core::region::Region>,
        log_group_name: String,
        log_stream_name: String,
    },
}

impl Default for MetricPublisherConfig {
    fn default() -> Self {
        Self::CloudWatchLogs {
            region: Default::default(),
            log_group_name: crate::cloudwatch::CloudWatchLogger::default_log_group(),
            log_stream_name: crate::cloudwatch::CloudWatchLogger::default_log_stream(),
        }
    }
}

impl MetricPublisherConfig {
    pub fn create_metric_publisher(&self) -> Arc<RwLock<dyn MetricPublisher>> {
        let publisher: Arc<RwLock<dyn MetricPublisher>> = match self {
            Self::Logger => Arc::new(RwLock::new(LoggerMetricPublisher::new())),
            Self::CloudWatchMetrics(maybe_region) => {
                let region = maybe_region.clone().unwrap_or_default();
                Arc::new(RwLock::new(CloudWatchMetricPublisher::new(&region)))
            }
            Self::CloudWatchLogs {
                region,
                log_group_name,
                log_stream_name,
            } => {
                let region = region.clone().unwrap_or_default();
                Arc::new(RwLock::new(CloudWatchLogger::with_log_stream(
                    log_stream_name.clone(),
                    log_group_name.clone(),
                    &region,
                )))
            }
        };
        publisher
    }
}
