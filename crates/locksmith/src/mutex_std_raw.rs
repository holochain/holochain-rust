use std::sync::{Mutex, RwLock};

pub type HcMutex<T> = Mutex<T>;
pub type HcRwLock<T> = RwLock<T>;
