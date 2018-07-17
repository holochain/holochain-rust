# Holochain in Rust
This is a beginning and architecture proposal of a second iteration of
Holochain build in Rust with the intention to have
1. some modules be compiled to WebAssembly to be reused in Holo's front-end part and
2. be able to use a somewhat mature WebAssembly interpreter like [wasmi](https://github.com/paritytech/wasmi) for a new type of Ribosome.

### Building for Android
Note: These instructions for building Holochain on Android are adapted from [here](https://mozilla.github.io/firefox-browser-architecture/experiments/2017-09-21-rust-on-android.html).

In order to get to libraries that can be linked against when building [HoloSqape](https://github.com/holochain/holosqape) for Android, you basically just need to setup up according targets for cargo.

Given that the Android SDK is installed, here are the steps to setting things up for building:

1. Install the Android tools:

    a. Install [Android Studio](https://developer.android.com/studio/)
    b. Open Android Studio and navigate to SDK Tools:
        - MacOS: `Android Studio > Preferences > Appearance & Behaviour > Android SDK > SDK Tools`
        - Linux: `Configure (gear) >  Appearance & Behavior > System Settings > Android SDK`
    c. Check the following options for installation and click OK:
        * Android SDK Tools
        * NDK
        * CMake
        * LLDB
    d. Get a beverage of your choice (or a full meal for that matter) why you wait for the lengthy download

1. Setup ANDROID_HOME env variable:

On MacOS

```bash
export ANDROID_HOME=/Users/$USER/Library/Android/sdk
```

Linux: (assuming you used defaults when installing Android Studio)

```bash
export ANDROID_HOME=$HOME/Android/Sdk
```

2. Create standalone NDKs (the commands below put the NDK in your home dir but you can put them where you like):

```bash
export NDK_HOME=$ANDROID_HOME/ndk-bundle
cd ~
mkdir NDK
${NDK_HOME}/build/tools/make_standalone_toolchain.py --api 26 --arch arm64 --install-dir NDK/arm64
${NDK_HOME}/build/tools/make_standalone_toolchain.py --api 26 --arch arm --install-dir NDK/arm
${NDK_HOME}/build/tools/make_standalone_toolchain.py --api 26 --arch x86 --install-dir NDK/x86
```

3. Add the following lines to your ```~/.cargo/config```:

```toml
[target.aarch64-linux-android]
ar = "<your $HOME value here>/NDK/arm64/bin/aarch64-linux-android-ar"
linker = "<your $HOME value here>/NDK/arm64/bin/aarch64-linux-android-clang"

[target.armv7-linux-androideabi]
ar = "<your $HOME value here>/NDK/arm/bin/arm-linux-androideabi-ar"
linker = "<your $HOME value here>/NDK/arm/bin/arm-linux-androideabi-clang"

[target.i686-linux-android]
ar = "<your $HOME value here>/NDK/x86/bin/i686-linux-android-ar"
linker = "<your $HOME value here>/NDK/x86/bin/i686-linux-android-clang"

```
(this toml file needs absolute paths, so you need to prefix the path with your home dir).

4. Now you can add those targets to your rust installation with:

```
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android
```

Finally, should now be able to build Holochain for Android with your chosen target, e.g.:

```
cd <holochain repo>
cargo build --target armv7-linux-androideabi --release
```

**NOTE:**  there is currently a problem in that `wabt` (which we use in testing as a dev dependency) won't compile on android, and the cargo builder compiles dev dependencies even though they aren't being used in release builds.  Thus as a work around, for the cargo build command above to work, you need to manually comment out the dev dependency section in both `core/Cargo.toml` and `core_api/Cargo.toml`

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

**NEVER RUN `make` ON ITS OWN UNLESS YOU KNOW WHAT YOU'RE DOING**

CI builds are happening on circle CI.

### Docker

The `docker` folder contains scripts to build and run docker images.

#### Running tests

Run:

```shell
. docker/run-test
```

#### Code style
There is a linter/formatter enforcing code style.

Run:

```shell
. docker/run-fmt
```

### holochain_101 mdbook

There is an [mdbook](https://github.com/rust-lang-nursery/mdBook) book on learning holochain at `doc/holochain_101`.

There is also a docker build that allows local build, serve, watch and live reload for the book.

From the root of the repo, run:

```shell
. docker/build-mdbook && . docker/run-mdbook
```

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
