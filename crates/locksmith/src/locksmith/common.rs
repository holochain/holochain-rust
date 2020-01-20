use crate::tracker::GuardTracker;
use parking_lot::{Mutex, MutexGuard};
use snowflake::ProcessUniqueId;
use std::{collections::HashMap, time::Duration};

lazy_static! {

    /// if a lock guard lives this long, it is assumed it will never die
    pub(crate) static ref IMMORTAL_TIMEOUT: Duration = Duration::from_secs(60);

    /// this should be a bit longer than IMMORTAL_TIMEOUT, so that locks don't timeout
    /// before all long-running guards are detected, in the case of a deadlock.
    /// (But NOT longer than try-o-rama's conductor timeout)
    pub(crate) static ref LOCK_TIMEOUT: Duration = Duration::from_secs(120);

    /// This is how often we check the elapsed time of guards
    pub(crate) static ref GUARD_WATCHER_POLL_INTERVAL: Duration = Duration::from_millis(1000);

    /// We filter out any guards alive less than this long
    pub(crate) static ref ACTIVE_GUARD_MIN_ELAPSED: Duration = Duration::from_millis(1000);

    /// Only report about no activity if this much time has passed
    pub(crate) static ref ACTIVE_GUARD_NO_ACTIVITY_INTERVAL: Duration = Duration::from_secs(10);

    static ref GUARDS: Mutex<GuardsMap> = Mutex::new(HashMap::new());
}

type GuardsMap = HashMap<ProcessUniqueId, GuardTracker>;

pub(crate) fn guards_guard<'a>() -> MutexGuard<'a, GuardsMap> {
    GUARDS
        .try_lock_for(Duration::from_secs(20))
        .expect("Guard-tracking mutex has been locked up for 20 seconds!")
}
