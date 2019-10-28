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

pub trait MetricPublisher {
    fn publish(&mut self, metric: &Metric);
}

pub struct StdoutMetricPublisher;

impl StdoutMetricPublisher {
    pub fn new() -> Self {
        StdoutMetricPublisher
    }
}
impl MetricPublisher for StdoutMetricPublisher {
    fn publish(&mut self, metric: &Metric) {
        println!("{} {}", metric.name, metric.value);
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn can_publish_to_stdout() {
        let publisher = StdoutMetricPublisher;
        let metric = Metric("latency", 100);

        publisher.publish(metric);
    }

}
