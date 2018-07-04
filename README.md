# Holochain in Rust
This is a beginning and architecture proposal of a second iteration of
Holochain build in Rust with the intention to have
1. some modules be compiled to WebAssembly to be reused in Holo's front-end part and
2. be able to use a somewhat mature WebAssembly interpreter like [wasmi](https://github.com/paritytech/wasmi) for a new type of Ribosome.

## Architecture
I've tried to resemble Redux in Rust and looked at [this code](https://github.com/rust-redux/rust-redux).

instance::Instance has a state::State which is the one global state with
sub-state slices for each module which are defined in each module respectively
(see src/agent/mod.rs, src/network/mod.rs and src/nucleus/mod.rs) and put
together in src/state.rs.

State is only read from the instance

```rust
instance.state().nucleus().dna()
```

and mutated by dispatching an action:

```rust
let entry = Entry{...};
instance.dispatch(state::Action::Agent(Commit(entry)));
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
    State {
        nucleus: ::nucleus::reduce(Rc::clone(&self.nucleus), action),
        agent: ::agent::reduce(Rc::clone(&self.agent), action)

    }
}
```

The module 'state' defines an action type (enum state::Action) that has values for
each sub-module. The modules define their sub-actions themselves and provide
their own sub-reducer function that handles those action types.

Since sub-module state slices are included in state::State as counted references (Rc\<AgentState>) the sub-module reducers can choose if they have the new state object (that the reducer returns) reference the same old sub-state slice (when the action did not affect the sub-state for instance) or if they clone the state, mutate it and return a different reference.

In module agent:

```rust
pub fn reduce(old_state: Rc<AgentState>, action: &_Action) -> Rc<AgentState> {
    match *action {
        _Action::Agent(ref agent_action) => {
            let mut new_state: AgentState = (*old_state).clone();
            match *agent_action {
                Action::Commit(ref entry) => {

                }
            }
            Rc::new(new_state)
        },
        _ => old_state
    }
}
```

With every module handling its state which is read-only for everything else and providing actions to be created from anywhere else that are processed through the reducer hierarchy I hope to decouple modules effectively. Actions being logged make already for a great debugging tool, if that is not enough, the state history could be stored and in a future debugging tool even switched back and forth (time-machine debugging for Holochain :D).

## Local development & testing

CI builds are happening on circle CI.

### Docker

The `docker` folder contains scripts to build and run docker images.

#### Running tests

Build:

`. docker/build-ubuntu`

Run:

`. docker/run-test`

#### Code style

There is a linter/formatter enforcing code style.

Build:

```
. docker/build-ubuntu
```

Run:

`. docker/run-fmt`

### holochain_101 mdbook

There is an [mdbook](https://github.com/rust-lang-nursery/mdBook) book on learning holochain at `doc/holochain_101`.

There is also a docker build that allows local build, serve, watch and live reload for the book.

From the root of the repo, run:

`$ . docker/build-mdbook && . docker/run-mdbook`

Once the book has built and is serving, visit `http://localhost:3000` in the browser.

You can edit the markdown files in `doc/holochain_101` and the book will live reload.

## License
[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](http://www.gnu.org/licenses/gpl-3.0)

Copyright (C) 2018, Holochain Trust

This program is free software: you can redistribute it and/or modify it under the terms of the license p
rovided in the LICENSE file (GPLv3).  This program is distributed in the hope that it will be useful, bu
t WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR
 PURPOSE.

**Note:** We are considering other 'looser' licensing options (like MIT license) but at this stage are u
sing GPL while we're getting the matter sorted out.
