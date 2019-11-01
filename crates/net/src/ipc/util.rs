//! This module holds util functions, such as constructing time related values

use std;

/// helper to get milliseconds since the unix epoch as an f64
pub fn get_millis() -> f64 {
    let epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let time = epoch.as_secs() as f64 * 1000.0;
    time + (f64::from(epoch.subsec_nanos()) / 1_000_000.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_fuzzy_millisecond_correctness() {
        let first = get_millis();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let second = get_millis();
        let diff = second - first;
        assert!(diff > 5.0 && diff < 100.0);
    }
}
