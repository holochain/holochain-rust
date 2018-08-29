use std;

/// helper to get milliseconds since the unix epoch as an f64
pub fn get_millis() -> f64 {
    let epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let time = epoch.as_secs() as f64 * 1000.0;
    time + (f64::from(epoch.subsec_nanos()) / 1_000_000.0)
}
