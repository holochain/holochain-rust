use crate::{cloudwatch::CloudWatchMetricPublisher, LoggerMetricPublisher, MetricPublisher};
use holochain_core_types::sync::HcRwLock;
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
    pub fn create_metric_publisher(&self) -> Arc<HcRwLock<dyn MetricPublisher>> {
        let publisher: Arc<HcRwLock<dyn MetricPublisher>> = match self {
            Self::Logger => Arc::new(HcRwLock::new(LoggerMetricPublisher::new())),
            Self::CloudWatch(maybe_region) => {
                let region = maybe_region.clone().unwrap_or_default();
                Arc::new(HcRwLock::new(CloudWatchMetricPublisher::new(&region)))
            }
        };
        publisher
    }
}
