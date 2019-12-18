//! Metric suppport for holochain. Provides metric representations to
//! sample, publish, aggregate, and analyze metric data.
use holochain_locksmith::RwLock;
use lazy_static;
use std::sync::Arc;

/// Represents a single sample of a numerical metric determined by `name`.
// TODO Consider renaming to Sample
#[derive(Debug, Clone, PartialEq)]
pub struct Metric {
    pub name: String,
    pub value: f64,
}

impl Metric {
    pub fn new(name: &str, value: f64) -> Self {
        Self {
            name: name.to_string(),
            value,
        }
    }
}

/// An object capable of publishing metric data.
pub trait MetricPublisher: Sync + Send {
    /// Publish a single metric.
    fn publish(&mut self, metric: &Metric);
}

/// The default metric publisher trait implementation
pub type DefaultMetricPublisher = crate::logger::LoggerMetricPublisher;

lazy_static! {
    pub static ref PUBLISHER: Arc<RwLock<crate::logger::LoggerMetricPublisher>> =
        Arc::new(RwLock::new(crate::logger::LoggerMetricPublisher));
}

/// Wraps a standard rust function with latency timing that is published to
/// $publisher upon completion of $f($args,*). The latency metric name will
/// be "$metric_prefix.latency".
#[macro_export]
macro_rules! with_latency_publishing {
    ($metric_prefix:expr, $publisher:expr, $f:expr, $($args:expr),* ) => {{
        let clock = std::time::SystemTime::now();

        let ret = ($f)($($args),*);
        let latency = clock.elapsed().unwrap().as_millis();

        let metric_name = format!("{}.latency", $metric_prefix);

        let metric = $crate::Metric::new(metric_name.as_str(), latency as f64);
        $publisher.write().unwrap().publish(&metric);
        ret
    }}
}

/// Composes a collection of publishers which are all called for one metric sample.
pub struct MetricPublishers(Vec<Arc<RwLock<dyn MetricPublisher>>>);

impl MetricPublisher for MetricPublishers {
    fn publish(&mut self, metric: &Metric) {
        for publisher in &self.0 {
            publisher.write().unwrap().publish(&metric);
        }
    }
}

impl MetricPublishers {
    pub fn new(publishers: Vec<Arc<RwLock<dyn MetricPublisher>>>) -> Self {
        MetricPublishers(publishers)
    }

    /// No-op metric publisher since the publisher list is empty
    pub fn empty() -> Self {
        Self::new(vec![])
    }
}

impl Default for MetricPublishers {
    fn default() -> Self {
        let publishers = vec![
            crate::config::MetricPublisherConfig::default_logger().create_metric_publisher(),
            crate::config::MetricPublisherConfig::default_cloudwatch_logs()
                .create_metric_publisher(),
        ];
        Self(publishers)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use std::sync::{Arc, RwLock};
    #[test]
    fn can_publish_to_logger() {
        let mut publisher = crate::logger::LoggerMetricPublisher;
        let metric = Metric::new("latency", 100.0);

        publisher.publish(&metric);
    }

    fn test_latency_fn(x: bool) -> bool {
        x
    }

    #[test]
    fn can_publish_latencies() {
        let publisher = Arc::new(RwLock::new(crate::logger::LoggerMetricPublisher));

        let ret = with_latency_publishing!("test", publisher, test_latency_fn, true);

        assert!(ret)
    }
}
