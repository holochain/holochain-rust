# Contributing

[![Project](https://img.shields.io/badge/project-holochain-blue.svg?style=flat-square)](http://holochain.org/)
[![Chat](https://img.shields.io/badge/chat-chat%2eholochain%2enet-blue.svg?style=flat-square)](https://chat.holochain.net)

As an Open Source project Holochain welcomes many sorts of contributions, bug-report & fixes, code contributes, documentation, tests, feedback and more.

## Social
We are committed to foster a vibrant thriving community, including growing a culture that breaks cycles of marginalization and dominance behavior. In support of this, some open source communities adopt [Codes of Conduct](http://contributor-covenant.org/version/1/3/0/).  We are still working on our social protocols, and empower each team to describe its own *Protocols for Inclusion*.  Until our teams have published their guidelines, please use the link above as a general guideline.

## Coordination

* Chat with us on our [chat server](https://chat.holochain.net) or [Gitter](https://gitter.im/metacurrency/holochain)

## Test Driven Development
We use **test driven development**. When you add a new function or feature, be sure to add both unit tests and integration tests to shows that it works.  Pull requests without tests will most-likely not be accepted!

## App Spec Driven Development
In adding significant changes and new features to Holochain, we follow a specific test-driven development protocol:
1. Start by creating a branch in this repository and modifying the example app in the app_spec directory to demonstrates an actual implementation of the use of the new feature, including tests that would pass if the feature were actually implemented.
1. Create a pull request on that branch for the development team to talk about and discuss the suggested change.  The PR triggers Continuous Integration tests which will initially fail.
1. Do any development necessary in core or hdk crates of this repo to actually implement the feature demonstrated in `app_spec`
1. Finally, when the feature is fully implemented, the CI tests should turn green and the branch can be merged.

In this way `app_spec` works as a living specification with example app to build against.

## Compiler warnings

Compilation warnings are NOT OK in shared/production level code.

Warnings have a nasty habit of piling up over time. This makes your code increasingly unpleasant for other people to work with.

CI MUST fail or pass, there is no use in the ever noisier "maybe" status.

If you are facing a warning locally you can try:

0. Fixing it
1. Using `#[allow(***)]` inline to surgically override a once-off issue
2. Proposing a global `allow` for a specific rule
  - this is an extreme action to take
  - this should only be considered if it can be shown that:
    - the issue is common (e.g. dozens of `#allow[***]`)
    - disabling it won't cause issues/mess to pile up elsewhere
    - the wider Rust community won't find our codebase harder to work with

If you don't know the best approach, please ask for help!

It is NOT OK to disable `deny` for warnings globally at the CI or makefile/nix level.

You can allow warnings locally during development by setting the `RUSTFLAGS` environment variable.

#### Code style
We use rust-fmt to enforce code style so that we don't spend time arguing about this.

Run the formatter with:

```shell
. docker/run-fmt
```
or

``` shell
make fmt
```

or

``` shell
nix-shell --run hc-fmt
```

## Continuous Integration

### CI configuration changes

Please also be aware that extending/changing the CI configuration can be very time consuming. Seemingly minor changes can have large downstream impact.

Some notable things to watch out for:

- Adding changes that cause the Travis cache to be dropped on every run
- Changing the compiler/lint rules that are shared by many people
- Changing versions of crates/libs that also impact downstream crates/repos
- Changing the nightly version of Rust used
- Adding/removing tools or external libs

The change may not be immediately apparent to you. The change may break the development environment on a different operating system, e.g. Windows.

At the same time, we do not want to catastrophise and stifle innovation or legitimate upgrades.

If you have a proposal to improve our CI config, that is great!

Please open a dedicated branch for the change in isolation so we can discuss the proposal together.

Please broadcast the proposal in chat to maximise visibility and the opportunity for everyone to respond.

It is NOT OK to change the behaviour of tests/CI in otherwise unrelated PRs. SOMETIMES it MAY be OK to change CI in a related PR, e.g. adding a new lib that your code requires. DO expect that a change like this will probably attract additional scrutiny during the PR review process, which is unfortunate but important.

Use your best judgement and respect that other people, across all timezones, rely on this repository remaining a productive working environment 24/7/365.

### Updating the CI Environment

The continuous integration (CI) suite executes the same `. docker/run-test` command that developers are encouraged to run.

What happens if I need to change that environment? E.g. what if I need a new system library dependency installed?

- Step 1 - Add the dependency to `docker/Dockerfile.ubuntu`

```dockerfile
RUN apt-get update && apt-get install --yes\
  # ... snip ...
  my-new-lib-here
```

- Step 2 - Build it

```shell
. docker/build-ubuntu
```

- Step 3 - Test it out

```shell
. docker/run-test
```

- Step 4 - Wait a minute! The CI environment is still using the old Dockerfile!

If your changes do not break the current environment, you can submit a separate Pull Request first, and once it is merged, the CI environment should be up-to-date for your code change Pull Request.

Otherwise, you will need to speak to an admin who can force merge your full changes after testing locally.

The continuous integration (CI) suite executes the same `. docker/run-test` command that developers are encouraged to run.

What happens if I need to change that environment? E.g. what if I need a new system library dependency installed?

- Step 1 - Add the dependency to `docker/Dockerfile.ubuntu`

```dockerfile
RUN apt-get update && apt-get install --yes\
  # ... snip ...
  my-new-lib-here
```

- Step 2 - Build it

```shell
. docker/build-ubuntu
```

- Step 3 - Test it out

```shell
. docker/run-test
```

- Step 4 - Wait a minute! The CI environment is still using the old Dockerfile!

If your changes do not break the current environment, you can submit a separate Pull Request first, and once it is merged, the CI environment should be up-to-date for your code change Pull Request.

Otherwise, you will need to speak to an admin who can force merge your full changes after testing locally.


## Git Hygiene
This section describes our practices and guidelines for using git and making changes to the repo.

* We use Github's pull requests as our code review tool
* We encourage any dev to comment on pull requests and we think of the pull request not as a "please approve my code" but as a space for co-developing, i.e. asynchronous "pair-coding" of a sort.
* We develop features on separate branches identified by the Github issue number, i.e. `124-my-new-feature`
* We use merge (not rebase) so that commits related to a ticket can be retroactively explored.
* In most repos development happens on a `develop` branch which gets merged to master when there's a release.

## License
The default licensing for our repos is currently [![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](http://www.gnu.org/licenses/gpl-3.0)
Copyright (C) 2018, Holochain Trust

**Note:** We are considering other 'looser' licensing options (like MIT license) but at this stage are using GPL while we're getting the matter sorted out.  See [this article](https://medium.com/holochain/licensing-needs-for-truly-p2p-software-a3e0fa42be6c) for some of our thinking on licensing for distributed application frameworks.

If you contribute code do so knowing that we may change the licensing from GPL to some other form like MIT without notification.
