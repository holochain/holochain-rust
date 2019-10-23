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
/// 
/// The lifetime represents how long the `Store` will be alive. This is
/// in place so that `Observer`s can depend on non-static data as long as it
/// outlive the `Store`.
/// 
/// Ideally they could depend on data that lives until they return `false`,
/// but that is too complicated for the borrow-checker.
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

#[cfg(test)]
mod tests {
    use super::{State, Store};
    #[derive(Clone, Debug, Default, Eq, PartialEq)]
    struct Counter(u32);

    #[derive(Clone, Debug, Eq, PartialEq)]
    enum CounterAction {
        Increment,
        Reset,
    }

    impl State for Counter {
        type Action = CounterAction;

        fn reduce(&mut self, action: &Self::Action) {
            use CounterAction::*;
            match action {
                Increment => self.0 = self.0.wrapping_add(1),
                Reset => self.0 = 0,
            }
        }
    }

    fn assert_recreatable<S, A>(store: &Store<S>)
    where
        S: State<Action = A> + std::fmt::Debug + Eq,
        A: Clone + std::fmt::Debug + Eq,
    {
        let recreated: Store<'_, S> = Store::from_history(store.history().to_owned());
        assert_eq!(recreated.state(), store.state());
        assert_eq!(recreated.history(), store.history());
    }

    #[test]
    fn can_reduce() {
        let mut store: Store<Counter> = Store::new();
        assert_eq!(*store.state(), Counter(0));
        assert_eq!(*store.history(), []);
        assert_recreatable(&store);

        store.dispatch(CounterAction::Increment);
        assert_eq!(*store.state(), Counter(1));
        assert_eq!(*store.history(), [CounterAction::Increment]);
        assert_recreatable(&store);

        store.dispatch(CounterAction::Increment);
        assert_eq!(*store.state(), Counter(2));
        assert_eq!(
            *store.history(),
            [CounterAction::Increment, CounterAction::Increment]
        );
        assert_recreatable(&store);

        store.dispatch(CounterAction::Reset);
        assert_eq!(*store.state(), Counter(0));
        assert_eq!(
            *store.history(),
            [
                CounterAction::Increment,
                CounterAction::Increment,
                CounterAction::Reset
            ]
        );
        assert_recreatable(&store);
    }

    #[test]
    fn can_observe() {
        let mut times_observed = 0;

        {
            let mut store: Store<Counter> = Store::new();

            store.observe(Box::new(|_state| {
                times_observed += 1;
                true
            }));

            assert_eq!(*store.state(), Counter(0));
            assert_eq!(*store.history(), []);
            assert_recreatable(&store);

            store.dispatch(CounterAction::Reset);
            assert_eq!(*store.state(), Counter(0));
            assert_eq!(*store.history(), [CounterAction::Reset]);
            assert_recreatable(&store);

            store.dispatch(CounterAction::Increment);
            assert_eq!(*store.state(), Counter(1));
            assert_eq!(
                *store.history(),
                [CounterAction::Reset, CounterAction::Increment]
            );
            assert_recreatable(&store);
        }

        assert_eq!(times_observed, 2);
    }

    #[test]
    fn observers_can_complete() {
        let mut times_observed_a = 0;
        let mut times_observed_b = 0;

        {
            let mut store: Store<Counter> = Store::new();

            store.observe(Box::new(|_state| {
                times_observed_a += 1;
                true
            }));
            store.observe(Box::new(|_state| {
                times_observed_b += 1;
                false
            }));

            assert_eq!(*store.state(), Counter(0));
            assert_eq!(*store.history(), []);
            assert_recreatable(&store);

            store.dispatch(CounterAction::Reset);
            assert_eq!(*store.state(), Counter(0));
            assert_eq!(*store.history(), [CounterAction::Reset]);
            assert_recreatable(&store);

            store.dispatch(CounterAction::Increment);
            assert_eq!(*store.state(), Counter(1));
            assert_eq!(
                *store.history(),
                [CounterAction::Reset, CounterAction::Increment]
            );
            assert_recreatable(&store);
        }

        assert_eq!(times_observed_a, 2);
        assert_eq!(times_observed_b, 1);
    }

    #[test]
    fn empty_history_equals_default() {
        let default: Store<'_, Counter> = Store::new();
        let empty: Store<'_, Counter> = Store::from_history(Vec::new());
        assert_eq!(default.state(), empty.state());
        assert_eq!(default.history(), empty.history());
    }
}
