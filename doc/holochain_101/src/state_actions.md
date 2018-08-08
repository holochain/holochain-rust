# State & Actions

Holochain uses a hybrid global/local state model.

In our bio mimicry terms the global state is for "short term memory" and local
state wraps references to "long term memory".

The global state is implemented as Redux style reducers. Any module can dispatch
an action to the global state. The action will be "reduced" to a new state tree
value by the modules responsible for each branch of the state tree. The return
values from a reduction can either be polled directly from the state tree in a
thread, or through a call/response model where the return value is a new action.

Actions are stateless/immutable data structures that are dispatched by modules
to communicate a request to do something potentially state changing. Everything
in the system should be either stateless or change state only in response to an
incoming action.

The global state is called "short term memory" because it is highly dynamic,
readily inspectable, and volatile. It does not survive indefinitely and is best
thought of as a cache of recent history.

Local state is implemented using actors to co-ordinate memory and threads in
Rust for external, persistent state. The classic example is a database
connection to the database that stores entries and headers. The db actor
receives read/write messages, and a reference to the sender is stored in the
global state.

## Actions

The `action` module defines 3 layers of nesting for actions:

- `ActionWrapper`: struct needed for ???, contains a unique ID and the `Action`
- `Action`: struct contains a unique ID for the action and the `Signal`
- `Signal`: enum of specific data to a given action, e.g. `Signal::Commit`

Processing an incoming action is a 3 step process:

0. Implement `reduce` to resolve and dispatch to a handler
0. Resolve the action signal to an appropriate handler
0. Implement handler logic

### Reduce

The `reduce` implementation is essentially copypasta. It handles resolving and
dispatching to a handler with a new state clone. The handler resolution and
dispatch logic should be split to facilitate clean unit testing.

```rust
pub fn reduce(
    old_state: Arc<FooState>,
    action: &Action,
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
) -> Arc<AgentState> {
  let handler = resolve_action_handler(action);
  match handler {
      Some(f) => {
          let mut new_state: FooState = (*old_state).clone();
          f(&mut new_state, &action, action_channel, observer_channel);
          Arc::new(new_state)
      }
      None => old_state,
  }
}
```

### Resolve an appropriate handler

The action handler should map signals to action handlers.

```rust
fn resolve_action_handler(
    action: &Action,
) -> Option<fn(&mut AgentState, &Action, &Sender<ActionWrapper>, &Sender<Observer>)> {
    match action.signal() {
        Signal::Commit(_, _) => Some(handle_commit),
        Signal::Get(_) => Some(handle_get),
        _ => None,
    }
}
```

### Implement the handlers

Each handler should respond to one action signal and mutate the relevant state.

The standard pattern is to maintain a `HashMap` of incoming actions against the
result of that action from the perspective of the current module. Each action
has a unique `id` internally so there will be no key collisions.

```rust
fn handle_foo(
    state: &mut FooState,
    action: &Action,
    _action_channel: &Sender<ActionWrapper>,
    _observer_channel: &Sender<Observer>,
) {
    let signal = action.signal();
    let bar = unwrap_to!(signal => Signal::Bar);

    // do something with bar...
    let result = bar.do_something();

    state
        .actions
        .insert(action.clone(), ActionResponse::Bar(result.clone()));
}
```

WARNING: Actions are reduced in a simple loop. Holochain will hang if you
dispatch and block on a new action while an outer action reduction is also
blocking, waiting for a response.

## Global state

`instance::Instance` has a `state::State` which is the one global state. Each
stateful module has a `state.rs` module containing sub-state slices.

See `src/agent/state.rs` and `src/nucleus/state.rs` and how they are put
together in `src/state.rs`.

State is read from the instance through relevant getter methods:

```rust
instance.state().nucleus().dna()
```

and mutated by dispatching an action:

```rust
let entry = Entry::new( ... );
let action = Action::new(&Signal::Commit(entry));
instance.dispatch(action);
```

Instance calls reduce on the state with the next action to consume:

```rust
pub fn consume_next_action(&mut self) {
    if self.pending_actions.len() > 0 {
        let action = self.pending_actions.pop_front().unwrap();
        self.state = self.state.clone().reduce(&action);
    }
}
```

The main reducer creates a new State object and calls the sub-reducers:

```rust
pub fn reduce(&mut self, action: &Action) -> Self {
    let mut new_state = State {
        nucleus: ::nucleus::reduce( ... ),
        agent: ::agent::reduce( ... )
    }

    new_state.history.insert(action_wrapper);
    new_state
}
```

Each incoming action is logged in the main state `history` to facilitate testing
and "time travel" debugging.

Sub-module state slices are included in `state::State` as counted references.

The sub-module reducer must choose to either:

- If mutations happen, return a cloned, mutated state slice with a new reference
- If no mutations happen, return the reference to the original state slice

The `reduce` copypasta above demonstrates this as the possible return values.

Redux in Rust code was used as a reference from [this repository](https://github.com/rust-redux/rust-redux).

## Local state

Coming Soon.
