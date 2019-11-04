use backtrace::Backtrace;
use snowflake::ProcessUniqueId as Puid;
use parking_lot::Mutex;
use std::{convert::{TryFrom, TryInto}, cmp, thread, time::Instant, collections::{BTreeMap, HashMap, hash_map}};

lazy_static! {
    pub static ref PATTERN_STATE: Mutex<LockPatternState> = Mutex::new(LockPatternState::default());
}

type DetectionError = String;

type Interval = (Instant, Instant);

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct LockInfo {
    lock_puid: Puid,
    thread_id: thread::ThreadId,
}

/// Tracks the entire lifetime of a mutex guard, from
/// acquisition to release
#[derive(Clone, Debug)]
struct LockLifetime {
    info: LockInfo,
    /// The timestamp of acquisition and release, respectively
    interval: Interval,
    backtraces: (Backtrace, Backtrace),
}

#[derive(Clone, Debug)]
struct LockAcquire {
    info: LockInfo,
    backtrace: Backtrace,
    time: Instant,
}

#[derive(Clone, Debug)]
struct LockRelease {
    info: LockInfo,
    backtrace: Backtrace,
    time: Instant,
}

#[derive(Clone, Debug)]
enum LockEvent {
    Acquire(LockAcquire),
    Release(LockRelease),
}

type EventPair = (LockAcquire, LockRelease);

struct LockSequence(Vec<LockLifetime>);

impl LockSequence {
    fn interval(&self) -> Interval {
        let initial: Interval = self.0.first().unwrap().interval;
        self.0.iter().map(|l| l.interval).fold(initial, |(lo, hi), (lo_, hi_)| (cmp::min(lo, lo_), cmp::max(hi, hi_)))
    }
}

impl TryFrom<EventPair> for LockLifetime {
    type Error = DetectionError;

    fn try_from(event_pair: EventPair) -> Result<LockLifetime, Self::Error> {
        let (acq, rel) =  event_pair;
        if acq.info != rel.info {
            return Err(format!("Lock event pair do not match up: {:?} != {:?}", acq.info, rel.info))
        }
        Ok(LockLifetime {
            info: acq.info,
            backtraces: (acq.backtrace, rel.backtrace),
            interval: (acq.time, rel.time),
        })
    }
}

#[derive(Default)]
pub struct LockPatternState {
    open_events: HashMap<thread::ThreadId, Vec<LockEvent>>,
    patterns: BTreeMap<Instant, LockSequence>,
}

impl LockPatternState {
    fn foldp(&mut self, event: LockEvent) -> Result<(), DetectionError> {
        let thread_id = thread::current().id();
        match self.open_events.entry(thread_id) {
            hash_map::Entry::Vacant(e) => {
                println!("INIT");
                e.insert(vec![event]);
            },
            hash_map::Entry::Occupied(mut e) => {
                let events = e.get_mut();
                {
                    println!("PUSH");
                    events.push(event);
                    match Self::check_completeness(&events)? {
                        Some(sequence) => {
                            println!("CLEARING");
                            events.clear();
                            let beginning = sequence.interval().0;
                            self.patterns.insert(beginning, sequence);
                        }
                        None => ()
                    }
                }
            }
        };
        Ok(())
    }

    fn events(&mut self) -> &Vec<LockEvent> {
        self.open_events.entry(thread::current().id()).or_insert(vec![])
    }

    fn analyze_patterns(&self) {
        // get patterns from 
    }

    fn check_completeness(events: &Vec<LockEvent>) -> Result<Option<LockSequence>, DetectionError> {
        let mut past: HashMap<LockInfo, LockAcquire> = HashMap::new();
        let mut sequence: BTreeMap<Instant, LockLifetime> = BTreeMap::new();
        let mut balance: u32 = 0;
        for event in events.iter().cloned() {
            match event {
                LockEvent::Acquire(acq) => {
                    balance += 1;
                    past.insert(acq.info.clone(), acq.clone());
                },
                LockEvent::Release(rel) => {
                    balance -= 1;
                    match past.get(&rel.info) {
                        Some(acq) => {
                            let lifetime = (acq.clone(), rel.clone()).try_into()?;
                            sequence.insert(acq.time, lifetime);
                        },
                        None => return Err("Encountered Release without matching Acquisition".into())
                    }
                }
            }
            println!("{}, {:?}", balance, sequence);
        }
        println!("FINAL: {}, {:?}", balance, sequence);
        if balance == 0 {
            // TODO: how not to clone?
            Ok(Some(LockSequence(sequence.values().cloned().collect())))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn acquire(puid: Puid, instant: Instant) -> LockEvent {
        LockEvent::Acquire(LockAcquire {
            info: LockInfo {
                lock_puid: puid,
                thread_id: thread::current().id()
            },
            time: instant,
            backtrace: Backtrace::new_unresolved()
        })
    }

    fn release(puid: Puid, instant: Instant) -> LockEvent {
        LockEvent::Release(LockRelease {
            info: LockInfo {
                lock_puid: puid,
                thread_id: thread::current().id()
            },
            time: instant,
            backtrace: Backtrace::new_unresolved()
        })
    }

    fn is_complete(state: &mut LockPatternState) -> bool {
        LockPatternState::check_completeness(state.events()).unwrap().is_some()
    }

    #[test]
    fn fold_events() {
        let lock1 = Puid::new();
        let lock2 = Puid::new();
        let _lock3 = Puid::new();
        let mut state = LockPatternState::default();
        assert_eq!(state.events().len(), 0);
        
        state.foldp(acquire(lock1, Instant::now())).unwrap();
        assert!(!is_complete(&mut state));
        assert_eq!(state.events().len(), 1);
        
        state.foldp(acquire(lock2, Instant::now())).unwrap();
        assert!(!is_complete(&mut state));
        assert_eq!(state.events().len(), 2);
        
        state.foldp(release(lock2, Instant::now())).unwrap();
        assert!(!is_complete(&mut state));
        assert_eq!(state.events().len(), 3);
        
        state.foldp(release(lock1, Instant::now())).unwrap();
        assert_eq!(state.events().len(), 0);
        assert_eq!(state.patterns.len(), 1);
    }
}