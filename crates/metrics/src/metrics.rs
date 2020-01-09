use chrono::prelude::*;
use crossbeam_channel::*;
use holochain_locksmith::RwLock;
/// Metric suppport for holochain. Provides metric representations to
/// sample, publish, aggregate, and analyze metric data.
use std::sync::Arc;

/// Represents a single sample of a numerical metric determined by `name`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub stream_id: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
    pub value: f64,
}

impl Metric {
    pub fn new<S: Into<String>, S2: Into<Option<String>>>(
        name: S,
        stream_id: S2,
        timestamp: Option<DateTime<Utc>>,
        value: f64,
    ) -> Self {
        Self {
            name: name.into(),
            stream_id: stream_id.into(),
            timestamp,
            value,
        }
    }

    pub fn new_timestamped_now<S: Into<String>, S2: Into<Option<String>>>(
        name: S,
        stream_id: S2,
        value: f64,
    ) -> Self {
        Self {
            name: name.into(),
            stream_id: stream_id.into(),
            timestamp: Some(Utc::now()),
            value,
        }
    }
}

/// Give a csv reader produce an iterator over metric.
/// Panics if any records are invalid from the csv source.
#[macro_export]
macro_rules! metrics_from_reader {
    ($read: expr) => {{
        $read.deserialize().map(|record| {
            let metric: Metric = record.unwrap();
            metric
        })
    }};
}

/// An object capable of publishing metric data.
pub trait MetricPublisher: Sync + Send {
    /// Publish a single metric.
    fn publish(&mut self, metric: &Metric);
}

/// WIP: Wraps another publisher and dedicates a processing thread to do actual publishing.
pub struct ChannelPublisher {
    sender: Sender<Metric>,
}

impl ChannelPublisher {
    pub fn new(mut metric_publisher: Box<dyn MetricPublisher>) -> Self {
        let (sender, receiver) = unbounded();
        let _join_handle: std::thread::JoinHandle<()> = std::thread::spawn(move || loop {
            match receiver.try_recv() {
                Ok(metric) => metric_publisher.publish(&metric),
                Err(TryRecvError::Disconnected) => break,
                Err(_) => (),
            }
        });

        Self { sender }
    }
}

impl MetricPublisher for ChannelPublisher {
    fn publish(&mut self, metric: &Metric) {
        self.sender.send(metric.clone()).unwrap();
    }
}

/// The default metric publisher trait implementation
pub type DefaultMetricPublisher = crate::logger::LoggerMetricPublisher;

/// Wraps a standard rust function with latency timing that is published to
/// $publisher upon completion of $f($args,*). The latency metric name will
/// be "$metric_prefix.latency".
#[macro_export]
macro_rules! with_latency_publishing {
    ($metric_prefix:expr, $publisher:expr, $f:expr) => {{
        let clock = std::time::SystemTime::now();

        let ret = $f();
        let latency = clock.elapsed().unwrap().as_millis();

        if latency == 0 {
            ret
        } else {
            let metric_name = format!("{}.latency", $metric_prefix);

            // TODO pass in stream id or not?
            let metric = $crate::Metric::new(metric_name.as_str(), None,
            Some(clock.into()), latency as f64);
            $publisher.write().unwrap().publish(&metric);
            ret
        }
    }};
    ($metric_prefix:expr, $publisher:expr, $f:expr, $($args:expr),+ ) => {{
        let clock = std::time::SystemTime::now();

        let ret = ($f)($($args),+);
        let latency = clock.elapsed().unwrap().as_millis();

        if latency == 0 {
            ret
        } else {
            let metric_name = format!("{}.latency", $metric_prefix);

            // TODO pass in stream id or not?
            let metric = $crate::Metric::new(metric_name.as_str(), None,
            Some(clock.into()), latency as f64);
            $publisher.write().unwrap().publish(&metric);
            ret
        }
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
        let timestamp = Utc.timestamp(1_500_000_000, 0);
        let metric = Metric::new("latency", None, Some(timestamp), 100.0);

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
