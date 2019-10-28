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
        trace!("{} {}", metric.name, metric.value);
    }
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

}
