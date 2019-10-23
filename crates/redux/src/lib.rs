/// An interface for self-reducing state
pub trait State: Default {
    /// The type of the actions behind reduced.
    type Action;

    fn reduce(&mut self, action: &Self::Action);
}

/// A function to be called exactly once per state change. Returns whether it
/// should remain registered.
pub type Observer<'a, S> = Box<dyn FnMut(&S) -> bool + Send + Sync + 'a>;

/// Wraps a state to ensure that it can only be updated through actions.
pub struct Store<'a, S: State> {
    state: S,
    history: Vec<S::Action>,
    observers: Vec<Observer<'a, S>>,
}

impl<S: State> Default for Store<'_, S> {
    fn default() -> Self {
        Self {
            state: Default::default(),
            history: Default::default(),
            observers: Default::default(),
        }
    }
}

impl<'a, S: State> Store<'a, S> {
    /// Creates a new `Store` using the `Default` implementation of `S`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Updates the state in accordance with `action` and notifies observers.
    pub fn dispatch(&mut self, action: S::Action) {
        self.state.reduce(&action);
        self.history.push(action);

        // Notify observers.
        // TODO: replace with `drain_filter` once it stabilizes
        let mut i = 0;
        while i < self.observers.len() {
            let keep = (&mut self.observers[i])(&self.state);
            if !keep {
                // Silence the must_use warning because
                // the function was called while it was in the vector.
                let _observer = self.observers.remove(i);
            } else {
                i += 1;
            }
        }
    }

    /// Registers an observer.
    ///
    /// Each time `dispatch` is called, `observer` will be called exactly once.
    /// If `observer` ever returns `false`, it will be dropped and will not be
    /// called on future calls to `dispatch`.
    pub fn observe(&mut self, observer: Observer<'a, S>) {
        self.observers.push(observer);
    }

    /// Returns the state owned by this `Store`.
    pub fn state(&self) -> &S {
        &self.state
    }

    /// Returns the set of all actions passed to `dispatch`, in order.
    pub fn history(&self) -> &[S::Action] {
        &self.history
    }

    /// Constructs a `Store` by replaying the actions in `history`.
    pub fn from_history(history: Vec<S::Action>) -> Self {
        let mut store = Self::new();
        history
            .iter()
            .for_each(|action| store.state.reduce(&action));
        store.history = history;
        store
    }
}
