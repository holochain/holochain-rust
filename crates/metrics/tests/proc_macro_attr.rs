//! This test aims at checking the usage of the latency attribute macro with the holochain's metric crate.

#[cfg(test)]
mod tests {
    // use super::*;
    use holochain_metrics::prelude::*;

    #[latency]
    pub fn test_latency() {
        ::std::thread::sleep(::std::time::Duration::from_millis(1001));
    }

    #[test]
    fn main_test() {
        eprintln!("Ahoy !");
    }
}
