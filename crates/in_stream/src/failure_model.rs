use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::Exp;
use scan_fmt::scan_fmt;
use std::time::{Duration, SystemTime};

const WS_FAILURE_MODEL_ENV_KEY: &str = "WS_FAILURE_MODEL";

#[derive(Clone, Debug)]
pub enum FailureState {
    Failing,
    NotFailing,
}

impl FailureState {
    pub fn invert(self) -> Self {
        match self {
            FailureState::Failing => FailureState::NotFailing,
            FailureState::NotFailing => FailureState::Failing,
        }
    }
}

#[derive(Debug)]
pub struct FailureModel {
    rng: StdRng,
    state: FailureState,
    next_switch_time: SystemTime,
    time_between_failures_dist: Exp<f64>,
    failure_duration_dist: Exp<f64>,
}

impl FailureModel {
    pub fn new(
        seed: u64,
        mean_time_between_failures: Duration,
        mean_failure_duration: Duration,
    ) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            state: FailureState::Failing, // this will switch on the first poll so we always start in non-failing mode
            next_switch_time: SystemTime::now(),
            time_between_failures_dist: Exp::new(1.0 / mean_time_between_failures.as_secs_f64())
                .unwrap(),
            failure_duration_dist: Exp::new(1.0 / mean_failure_duration.as_secs_f64()).unwrap(),
        }
    }

    pub fn new_from_env_vars() -> Result<Self, String> {
        match std::env::var(WS_FAILURE_MODEL_ENV_KEY) {
            Ok(s) => {
                let (seed, mtbf, mfd) = scan_fmt!(&s, "({f}, {f}, {f})", u64, u64, u64)
                .map_err(|_| String::from("Invalid value for WS_FAILURE_MODEL. Must be 3-tuple of (seed, mean_ms_between_error_bursts, mean_ms_burst_length"))?;
                Ok(FailureModel::new(
                    seed,
                    Duration::from_millis(mtbf),
                    Duration::from_millis(mfd),
                ))
            }
            Err(_) => Err(String::from("No env var provided")),
        }
    }

    // returns the state of the model at the current moment in time
    // also updates the model to predict the next switch
    pub fn poll(&mut self) -> FailureState {
        let now = SystemTime::now();
        while self.next_switch_time < now {
            self.next_switch_time = self.next_switch_time + self.time_to_next_switch();
            self.state = self.state.clone().invert();
        }
        self.state.clone()
    }

    fn time_to_next_switch(&mut self) -> Duration {
        match self.state {
            FailureState::Failing => {
                // here the time to switch is based on the mean_failure_duration
                Duration::from_secs_f64(self.rng.sample(self.time_between_failures_dist))
            }
            FailureState::NotFailing => {
                // here the time to switch is based on the mean_time_between_failures
                Duration::from_secs_f64(self.rng.sample(self.failure_duration_dist))
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    // use super::*;
    // use std::thread::sleep;

    // #[test]
    // fn test_failure_model_can_poll() {
    // 	let mut model = FailureModel::new(0, Duration::from_millis(1000), Duration::from_millis(100));
    // 	loop {
    // 		model.poll();
    // 		println!("{:?}", model.state);
    // 		sleep(Duration::from_millis(100));
    // 	}
    // }
}
