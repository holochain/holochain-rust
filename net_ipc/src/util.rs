use ::std;

pub fn get_millis () -> f64 {
    let epoch = std::time::SystemTime::now().duration_since(
        std::time::UNIX_EPOCH).unwrap();
    let time = epoch.as_secs() as f64 * 1000.0;
    let time = time + (epoch.subsec_nanos() as f64 / 1000000.0);
    time
}
