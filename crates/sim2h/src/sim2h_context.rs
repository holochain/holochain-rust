use std::sync::Arc;
use lib3h_crypto_api::CryptoSystem;
use futures::{
    future::{Future, FutureExt},
    executor::ThreadPoolBuilder,
};

mod state;
use state::*;

/// spawn task fn lets us abstract the executor implementation
pub type Sim2hContextSpawnFn = Arc<dyn Fn(futures::future::BoxFuture<'static, ()>) + 'static + Send + Sync>;

/// cheaply clone-able context object that lets us share our
/// task spawning capabilities, crypto system, state, etc
pub struct Sim2hContext {
    spawn_fn: Sim2hContextSpawnFn,
    crypto: Box<dyn CryptoSystem>,
    state: Sim2hStateRef,
}

/// a reference to the sim2h context
pub type Sim2hContextRef = Arc<Sim2hContext>;

impl Sim2hContext {
    /// create a new sim2h context instance
    pub fn new(
        spawn_fn: Sim2hContextSpawnFn,
        crypto: Box<dyn CryptoSystem>,
    ) -> Sim2hContextRef {
        Arc::new(Self {
            spawn_fn: spawn_fn.clone(),
            crypto: crypto.box_clone(),
            state: Sim2hState::new(spawn_fn, crypto),
        })
    }

    /// spawn a new future task into our executor
    pub fn spawn<F>(&self, future: F)
        where F: Future<Output = ()> + Send + 'static
    {
        (self.spawn_fn)(future.boxed());
    }

    /// access the crypto system for doing crypto stuff
    pub fn crypto(&self) -> &dyn CryptoSystem {
        self.crypto.as_ref()
    }

    /// read-only state access
    pub fn state(&self) -> &Sim2hStateRef {
        &self.state
    }

    /// mutable state access
    pub fn state_mut(&mut self) -> &Sim2hStateRef {
        &mut self.state
    }
}

/// builds a generic cpu-count thread pool for sim2h
pub fn task_context_thread_pool(crypto: Box<dyn CryptoSystem>) -> Sim2hContextRef {
    let mut builder = ThreadPoolBuilder::new();
    let pool = builder
        .name_prefix("sim2h-context-thread-pool-")
        .create()
        .expect("error creating futures thread pool");
    let spawn_fn: Sim2hContextSpawnFn = Arc::new(move |future| {
        pool.spawn_ok(future);
    });
    Sim2hContext::new(spawn_fn, crypto)
}
