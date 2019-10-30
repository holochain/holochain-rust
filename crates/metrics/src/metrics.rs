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

pub trait MetricPublisher: Sync + Send {
    fn publish(&mut self, metric: &Metric);
}

#[derive(Debug, Clone)]
pub struct LoggerMetricPublisher;

impl LoggerMetricPublisher {
    pub fn new() -> Self {
        Self
    }
}

pub type DefaultMetricPublisher = LoggerMetricPublisher;

impl MetricPublisher for LoggerMetricPublisher {
    fn publish(&mut self, metric: &Metric) {
        debug!("{} {}", metric.name, metric.value);
    }
}

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

#[cfg(test)]
mod test {

    use super::*;
    #[test]
    fn can_publish_to_logger() {
        let mut publisher = LoggerMetricPublisher;
        let metric = Metric::new("latency", 100.0);

        publisher.publish(&metric);
    }

    fn test_latency_fn(x: bool) -> bool {
        x
    }

    #[test]
    fn can_publish_latencies() {
        let publisher = std::sync::Arc::new(std::sync::RwLock::new(LoggerMetricPublisher));

        let ret = with_latency_publishing!("test", publisher, test_latency_fn, true);

        assert!(ret)
    }

}
