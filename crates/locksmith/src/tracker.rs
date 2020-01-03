use crate::{
    common::{
        guards_guard, ACTIVE_GUARD_MIN_ELAPSED, ACTIVE_GUARD_NO_ACTIVITY_INTERVAL,
        GUARD_WATCHER_POLL_INTERVAL, IMMORTAL_TIMEOUT,
    },
    error::LockType,
};
#[cfg(feature = "sync_backtrace_capture")]
use backtrace::Backtrace;
use snowflake::ProcessUniqueId;
use std::{
    thread,
    time::{Duration, Instant},
};

pub(crate) struct GuardTracker {
    pub(crate) puid: ProcessUniqueId,
    pub(crate) created: Instant,
    #[cfg(feature = "sync_backtrace_capture")]
    pub(crate) backtrace: Backtrace,
    pub(crate) lock_type: LockType,
    pub(crate) immortal: bool,
    pub(crate) annotation: Option<String>,
}

impl GuardTracker {
    pub fn new(puid: ProcessUniqueId, lock_type: LockType) -> Self {
        Self {
            puid,
            lock_type,
            created: Instant::now(),
            #[cfg(feature = "sync_backtrace_capture")]
            backtrace: Backtrace::new_unresolved(),
            immortal: false,
            annotation: None,
        }
    }

    pub fn report_and_update(&mut self) -> Option<(i64, String)> {
        let elapsed = Instant::now().duration_since(self.created);
        if elapsed > *ACTIVE_GUARD_MIN_ELAPSED {
            let elapsed_ms = elapsed.as_millis() as i64;
            if !self.immortal && elapsed > *IMMORTAL_TIMEOUT {
                self.immortalize();
            }
            let lock_type_str = format!("{:?}", self.lock_type);
            let report = if self.immortal {
                format!(
                    "{:<6} {:<13} {:>12} [!!!]",
                    lock_type_str, self.puid, elapsed_ms
                )
            } else {
                format!("{:<6} {:<13} {:>12}", lock_type_str, self.puid, elapsed_ms)
            };
            Some((elapsed_ms, report))
        } else {
            None
        }
    }

    pub fn report_header() -> String {
        format!("{:6} {:^13} {:>12}", "KIND", "PUID", "ELAPSED (ms)")
    }

    fn immortalize(&mut self) {
        if self.immortal {
            return;
        }
        self.immortal = true;
        #[cfg(feature = "sync_backtrace_capture")]
        self.backtrace.resolve();
        let annotation = self
            .annotation
            .as_ref()
            .map(|a| format!("\nAnnotation: {}\n", a))
            .unwrap_or_default();
        #[cfg(feature = "sync_backtrace_capture")]
        error!(
            r"

        !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
        !!! IMMORTAL LOCK GUARD FOUND !!!
        !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

{type:?} guard {puid} lived for > {time} seconds.{annotation}
Backtrace at the moment of guard creation follows:

{backtrace:?}",
            type=self.lock_type,
            puid=self.puid,
            time=IMMORTAL_TIMEOUT.as_secs(),
            annotation=annotation,
            backtrace=self.backtrace
        );
        #[cfg(not(feature = "sync_backtrace_capture"))]
        error!(
            r"

        !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
        !!! IMMORTAL LOCK GUARD FOUND !!!
        !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

{type:?} guard {puid} lived for > {time} seconds.{annotation}",
            type=self.lock_type,
            puid=self.puid,
            time=IMMORTAL_TIMEOUT.as_secs(),
            annotation=annotation
        );
    }
}

pub fn spawn_locksmith_guard_watcher() {
    debug!("spawning locksmith_guard_watcher");
    let _ = thread::Builder::new()
        .name(format!(
            "locksmith_guard_watcher/{}",
            ProcessUniqueId::new().to_string()
        ))
        .spawn(move || {
            let mut inactive_for = Duration::from_millis(0);
            loop {
                let mut reports: Vec<(i64, String)> = {
                    guards_guard()
                        .values_mut()
                        .filter_map(|gt| gt.report_and_update())
                        .collect()
                };
                if reports.len() > 0 {
                    inactive_for = Duration::from_millis(0);
                    reports.sort_unstable_by_key(|(elapsed, _)| -*elapsed);
                    let num_active = reports.len();
                    let lines: Vec<String> =
                        reports.into_iter().map(|(_, report)| report).collect();
                    let output = lines.join("\n");
                    debug!(
                        "tracking {} active guard(s) alive for > {}ms:\n{}\n{}",
                        num_active,
                        ACTIVE_GUARD_MIN_ELAPSED.as_millis(),
                        GuardTracker::report_header(),
                        output
                    );
                } else {
                    inactive_for += *GUARD_WATCHER_POLL_INTERVAL;
                    if inactive_for > *ACTIVE_GUARD_NO_ACTIVITY_INTERVAL {
                        debug!(
                            "no active guards alive > {:?}ms for the last {:?} seconds",
                            ACTIVE_GUARD_MIN_ELAPSED.as_millis(),
                            ACTIVE_GUARD_NO_ACTIVITY_INTERVAL.as_secs(),
                        );
                        inactive_for = Duration::from_millis(0);
                    }
                }

                thread::sleep(*GUARD_WATCHER_POLL_INTERVAL);
            }
        });
}
