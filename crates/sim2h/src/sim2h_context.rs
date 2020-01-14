use futures::{
    executor::ThreadPoolBuilder,
    future::{Future, FutureExt},
};
use lib3h_crypto_api::CryptoSystem;
use std::sync::Arc;

mod state;
pub use state::*;

/// spawn task fn lets us abstract the executor implementation
pub type Sim2hContextSpawnFn =
    Arc<dyn Fn(futures::future::BoxFuture<'static, ()>) + 'static + Send + Sync>;

/// we need these split out, because state needs access to them
pub struct Sim2hContextInner {
    pub spawn_fn: Sim2hContextSpawnFn,
    pub crypto: Box<dyn CryptoSystem>,
}

impl Clone for Sim2hContextInner {
    fn clone(&self) -> Self {
        Self {
            spawn_fn: self.spawn_fn.clone(),
            crypto: self.crypto.box_clone(),
        }
    }
}

pub struct DeleteMe<'lt> {
    delete_me: futures::lock::MutexGuard<'lt, Sim2hState>,
}

impl<'lt> DeleteMe<'lt> {
    pub fn read(&self) -> &Sim2hState {
        &self.delete_me
    }

    pub fn write(&mut self) -> &mut Sim2hState {
        &mut self.delete_me
    }
}

/// cheaply clone-able context object that lets us share our
/// task spawning capabilities, crypto system, state, etc
pub struct Sim2hContext {
    inner: Sim2hContextInner,
    #[allow(dead_code)]
    state: Sim2hStateRef,
}

/// a reference to the sim2h context
pub type Sim2hContextRef = Arc<Sim2hContext>;

impl Sim2hContext {
    /// create a new sim2h context instance
    pub fn new(
        spawn_fn: Sim2hContextSpawnFn,
        crypto: Box<dyn CryptoSystem>,
        state: Sim2hState,
    ) -> Sim2hContextRef {
        let inner = Sim2hContextInner { spawn_fn, crypto };
        let state = Sim2hStateMutex::new(inner.clone(), state);
        Arc::new(Self { inner, state })
    }

    #[allow(dead_code)]
    /// spawn a new future task into our executor
    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        (self.inner.spawn_fn)(future.boxed());
    }

    #[allow(dead_code)]
    /// access the crypto system for doing crypto stuff
    pub fn crypto(&self) -> &dyn CryptoSystem {
        self.inner.crypto.as_ref()
    }

    #[allow(clippy::borrowed_box)]
    /// some apis need the box around it... prefer `crypto()` when possible
    pub fn box_crypto(&self) -> &Box<dyn CryptoSystem> {
        &self.inner.crypto
    }

    /// DELETE ME - temporary direct access to state for iteration
    pub fn delete_me(&self) -> DeleteMe {
        DeleteMe {
            delete_me: self.state.delete_me_block_lock(),
        }
    }
}

/// builds a generic cpu-count thread pool for sim2h
pub fn sim2h_context_thread_pool(
    crypto: Box<dyn CryptoSystem>,
    state: Sim2hState,
) -> Sim2hContextRef {
    let mut builder = ThreadPoolBuilder::new();
    let pool = builder
        .name_prefix("sim2h-context-thread-pool-")
        .create()
        .expect("error creating futures thread pool");
    let spawn_fn: Sim2hContextSpawnFn = Arc::new(move |future| {
        pool.spawn_ok(future);
    });
    Sim2hContext::new(spawn_fn, crypto, state)
}
