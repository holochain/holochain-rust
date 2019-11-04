use crate::{
    cloudwatch::CloudWatchMetricPublisher, logger::LoggerMetricPublisher, MetricPublisher,
};
use holochain_locksmith::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MetricPublisherConfig {
    Logger,
    CloudWatch(Option<rusoto_core::region::Region>),
}

impl Default for MetricPublisherConfig {
    fn default() -> Self {
        Self::Logger
    }
}

impl MetricPublisherConfig {
    pub fn create_metric_publisher(&self) -> Arc<RwLock<dyn MetricPublisher>> {
        let publisher: Arc<RwLock<dyn MetricPublisher>> = match self {
            Self::Logger => Arc::new(RwLock::new(LoggerMetricPublisher::new())),
            Self::CloudWatch(maybe_region) => {
                let region = maybe_region.clone().unwrap_or_default();
                Arc::new(RwLock::new(CloudWatchMetricPublisher::new(&region)))
            }
        };
        publisher
    }
}
