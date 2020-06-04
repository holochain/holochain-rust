# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.49-alpha1] - 2020-05-28

### Added

### Changed

### Deprecated

### Removed
- Reverted back to v0.0.47-alpha1. [#2192](https://github.com/holochain/holochain-rust/pull/2192)
- update futures crate because of dependency issues.[#2192](https://github.com/holochain/holochain-rust/pull/2192)

### Fixed

### Security

## [0.0.49-alpha1] - 2020-05-14

### Added

- added an optimization of not querying the DHT for get links requests that are placed on a node's own agent id because we must be responsible for holding that part of the DHT. [#2189](https://github.com/holochain/holochain-rust/pull/2189)
- added an optimization for sim2h to not re-fetch entries we've fetched within the last second. [#2185](https://github.com/holochain/holochain-rust/pull/2185)

### Changed

### Deprecated

### Removed

### Fixed

- two conductor fixes during error cases (like base hasn't arrived yet) in hold_aspects request: [#2184](https://github.com/holochain/holochain-rust/pull/2184)
    - stop incorrectly recording aspects as held
    - stop locking up the future
- because sim2h doesn't get error messages from a hold_aspect request it must also not record aspects sent in those messages as held, and thus the conductor must explicitly send back a list of aspects held after a hold_aspect request [#2184](https://github.com/holochain/holochain-rust/pull/2184)
- update futures crate because of dependency issues.  [#2188](https://github.com/holochain/holochain-rust/pull/2188)

### Security

## [0.0.47-alpha1] - 2020-04-09

### Added

### Changed

### Deprecated

### Removed

### Fixed

- futures bug that caused CPU over-consumption [#2175](https://github.com/holochain/holochain-rust/pull/2175)
- validation bug that treated network timeout as a validation failure instead of a retry [#2176](https://github.com/holochain/holochain-rust/pull/2176)

### Security

## [0.0.46-alpha1] - 2020-03-27

### Added

### Changed

- Adds `--uuid` flag to `hc hash`, allowing a UUID to be specified which will alter the hash [PR#2161](https://github.com/holochain/holochain-rust/pull/2161)
- Adds `--files` flag to `hc sim2h-client`, which when set prints JSON blobs to multiple files named by space hash (the previous default behavior), and when unset prints a single JSON blob to stdout for easy parsing by script [PR#2161](https://github.com/holochain/holochain-rust/pull/2161)

### Deprecated

### Removed

### Fixed
- Log made a big more quiet by shifting errors into Debug log level
- IOError bug on handle_fetch_entry [PR#2148](https://github.com/holochain/holochain-rust/pull/2148)
- Bug where UpdateAspects were handled incorrectly
- Bugs causing validation timeouts [#2159](https://github.com/holochain/holochain-rust/pull/2159)  [#2169](https://github.com/holochain/holochain-rust/pull/2169)
- Bugs entry update [PR#2170](https://github.com/holochain/holochain-rust/pull/2170) &  [PR#2153](https://github.com/holochain/holochain-rust/pull/2153)

### Security

## [0.0.45-alpha1] - 2020-03-13

### Added
- Adds tokio tracing to sim2h_server. [Description](https://holo.hackmd.io/@c5lIpp4ET0OJJnDT3gzilA/SyRm2YoEU). Also check `sim2h_server --help` for usage instructions.
- Adds the notion of a manager to trycp_server so that we can dynamically manage pools of available nodes for test runs in final-exam  [PR#2123](https://github.com/holochain/holochain-rust/pull/2123)

### Changed
- new_relic is behind a feature flag `new-relic`.
### Deprecated

### Removed
- Older rust-tracing traces.

### Fixed

- Many bugs fixed to get_links [PR#2150](https://github.com/holochain/holochain-rust/pull/2150) [PR#2148](https://github.com/holochain/holochain-rust/pull/2148)

### Security

## [0.0.44-alpha3] - 2020-03-03

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.44-alpha2] - 2020-03-03

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.44-alpha1] - 2020-03-02

### Added

- Added command to sim2h wire protocol for getting live debug info [#2128](https://github.com/holochain/holochain-rust/pull/2128)
- Added an environment variable (HC_IGNORE_SIM2H_URL_PROPERTY) which overrides DNA sim2h_url value for running conductors in test modes

### Changed

- Changed Pagination to have different types [#2110](https://github.com/holochain/holochain-rust/pull/2110)
- Link matches are not based on regex anymore [#2133](https://github.com/holochain/holochain-rust/pull/2133)

### Deprecated

### Removed

### Fixed
- Fixes dropped join messages in sim2h that was blocking scaling [#2137](https://github.com/holochain/holochain-rust/pull/2137)
- Make Holochain (i.e. Sim2hWorker) work offline again (that is without being connected to Sim2h) [#2119](https://github.com/holochain/holochain-rust/pull/2119)
- Fixing wire message resilience to connection drops via receipts [#2120](https://github.com/holochain/holochain-rust/pull/2120)
- Fixed `panic!("entry/aspect mismatch - corrupted data?")` [#2135](https://github.com/holochain/holochain-rust/pull/2135)

### Security

## [0.0.43-alpha3] - 2020-02-10

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.43-alpha2] - 2020-02-10

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.43-alpha1] - 2020-02-09

### Added
- Added pagination option to get_links [#2092](https://github.com/holochain/holochain-rust/pull/2092)
- Added sort order option to get_links [#2100](https://github.com/holochain/holochain-rust/pull/2100)
- Removed networking support for sim1h [#2101](https://github.com/holochain/holochain-rust/pull/2101)
- Removed networking support for n3h [#2101](https://github.com/holochain/holochain-rust/pull/2101)

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.42-alpha5] - 2020-01-07

### Added

### Changed

### Deprecated

### Removed

### Fixed

- Fixes a panic in the conductor that happens when a get links times out [#2046](https://github.com/holochain/holochain-rust/pull/2046)
- Fixes a problem in MacOS with establishing network connections [#2047](https://github.com/holochain/holochain-rust/pull/2047)

### Security

## [0.0.42-alpha4] - 2020-01-07

### Added

### Changed

### Deprecated

### Removed

### Fixed

- Websocket Connection Error on macOs

### Security

## [0.0.42-alpha3] - 2020-01-05

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.42-alpha2] - 2020-01-02

### Added

### Changed

### Deprecated

### Removed

### Fixed

- Fixes bugs in the sim2 connecting handling. Backoff timing and reset was broken.

### Security

## [0.0.42-alpha1] - 2019-12-30

### Added

### Changed

- `hc` now passes arguments to bash at runtime [#2019](https://github.com/holochain/holochain-rust/pull/2019).
- `artifact` in `.hcbuild` now evaluates bash strings and does not force relative paths [#2020](https://github.com/holochain/holochain-rust/pull/2020)

### Deprecated

### Removed

### Fixed

### Security

## [0.0.41-alpha4] - 2019-12-20

### Added

- Added `print-metric-stats` and `print-cloudwatch-metrics` commands to `holochain_metrics` [#1972](https://github.com/holochain/holochain-rust/pull/1972).

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.41-alpha3] - 2019-12-19

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.41-alpha2] - 2019-12-19

### Added
- Adds support for setting the agent name and id via a parameter with hc run by calling `hc run --agent-name MyAgentName`. The `%agent_id` is generated from the name. This allows multiple hc run conductors to be used on the same machine. It will be overwritten by the `HC_AGENT` environment variable.
- Adds support for sim2h with hc run by calling `hc run --networked sim2h --sim2h-server wss://localhost:9000`.
- Adds support for [hApp-bundles](https://github.com/holochain/holoscape/tree/master/example-bundles) to `hc run`. This enables complex hApp setups with multiple DNAs and bridges to be easily run during development without having to write/maintain a conductor config file. [#1939](https://github.com/holochain/holochain-rust/pull/1939)
- Adds ability to validate entries with full chain validation when author is offline [#1932](https://github.com/holochain/holochain-rust/pull/1932)
- Adds a conductor level stats-signal that sends an overview of instance data (number of held entries etc.) over admin interfaces. [#1954](https://github.com/holochain/holochain-rust/pull/1954)
- Adds parameters to conductor RPC function `debug/state_dump` to select portions of the state to be send instead of always receiving the full dump (which can get big if the instance holds many entries). [#1954](https://github.com/holochain/holochain-rust/pull/1954)
- Added new docker boxes dedicated to faster CI tasks through incremental compilation
- Added `CARGO_CACHE_RUSTC_INFO=1` to nix shell

### Changed

- data sent via jsonrpc to the conductor interface for agent/sign, agent/encrypt and agent/decrypt must now be base64 encoded
- circleci config now uses version 2.1 syntax
- added the `-x` flag to several nix-shell commands
- using `command -v` instead of `which` in app spec `build_and_test.sh`
- standardised all app (proc) spec commands into a single paramaterised command `hc-test-app-spec`
- updated to holonix `v0.0.54`
- `$CARGO_TARGET_DIR` is now set explicitly in the nix shell hook
- renamed `hc-conductor-wasm-install` to `hc-conductor-wasm-bindgen-install`
- core `shellHook` can now override holonix `shellHook`
- several `--target-dir` flags are removed in favour of `$CARGO_TARGET_DIR`
- the passphrase hashing config is now set to faster and less secure parameters to reduce the start-up time of conductors a lot, esp. on slow devices. (will become a setting the user can choose in the future - faster and less secure config is fine for now and throughout alpha and beta) [#1986](https://github.com/holochain/holochain-rust/pull/1986)

### Deprecated

### Removed

### Fixed

- paths in cluster test are no longer hardcoded in a way that breaks `$CARGO_TARGET_DIR`
- `cli` and `conductor` are now both uninstalled again after running app spec tests
- Fixes a panic in the sim2h server that can happen if the last node of a space leaves just as a second node connects. [#1977](https://github.com/holochain/holochain-rust/pull/1977)

### Security

## [0.0.40-alpha1] - 2019-12-01

### Added

- Pruning the State of old/stale/history data to prevent it from using up a slowly but infinitely growing amount of memory. [#1920](https://github.com/holochain/holochain-rust/pull/1920)

### Changed

- Exchanged [vanilla thread-pool](https://docs.rs/threadpool/1.7.1/threadpool/) with the futures executor thread-pool [from the futures crate](https://docs.rs/futures/0.3.1/futures/executor/index.html). This enables M:N Future:Thread execution which is much less wasteful than having a thread per future. Number of threads in the pool is kept at the default (of that crate) of number of CPUs. [#1915](https://github.com/holochain/holochain-rust/pull/1915)
- Replace naive timeout implementation (for network queries / direct messages) that uses a thread per timeout with a scheduled job that polls the State and sends timeout actions when needed (reduces number of used threads and thus memory footprint) [#1916](https://github.com/holochain/holochain-rust/pull/1916).
- Use the [im crate](https://docs.rs/im/14.0.0/im/) for `HashMap`s and `HashSet`s used in the redux State. This makes cloning the state much cheaper and improves over-all performance. [#1923](https://github.com/holochain/holochain-rust/pull/1923)

### Deprecated

### Removed

### Fixed

### Security

## [0.0.39-alpha4] - 2019-11-25

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.39-alpha3] - 2019-11-25

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.39-alpha2] - 2019-11-25

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.39-alpha1] - 2019-11-25

### Added

- Adds a retry if a net worker cannot be spawned on startup [#1870](https://github.com/holochain/holochain-rust/pull/1870)
- Add hdk::version_hash, returning MD5 hash of HDK build environment [#1869](https://github.com/holochain/holochain-rust/pull/1869)
- Add --info option to conductor to return info on the version including HDK_VERSION & HASH as well as GIT_HASH & GIT_BRANCH if the binary was compiled from a git repo [1902](https://github.com/holochain/holochain-rust/pull/1902)
- Ability to set storage backend for new instances over RPC [#1900](https://github.com/holochain/holochain-rust/pull/1900)
- Tracing of HDK API function calls within a zome function call to be used for debugging and in Holoscape's debug view [#1885](https://github.com/holochain/holochain-rust/pull/1885)

### Changed

- Several improvements to gossip related code, both in sim2h server and core [#1884](https://github.com/holochain/holochain-rust/pull/1884/files):
  * Sim2h server will not just randomly pick a node to fill missing aspects, but it caches the information which aspects are missing for which node and will not ask a node about an aspect it doesn't have (gets rid of the `EntryNotFoundLocally` error).
  * In core's list responses: merge authoring list into the gossip list so sim2h has gossip sources that are the authors of entry aspects.
  * Clear sim2h server's caches about nodes when they disconnect. Also forget the whole space when the last node disconnectse.
- `DhtStore::holding_list` which stored only the hashes of entries being held got changed to `DhtStore::holding_map` which is a map of entry address to set of aspect addresses so we know explicitly which aspects are held for each entry. This helps debugging (and already revealed a bug which is fixed in this version too) and enabled several simplifications in core logic. [1904](https://github.com/holochain/holochain-rust/pull/1904)

### Deprecated

### Removed

### Fixed

- Fix lots of deadlocks by managing threads and encapsulating locks [#1852](https://github.com/holochain/holochain-rust/pull/1852)
- Have sim2h let go of nodes if the connection got lost because of an error [#1877](https://github.com/holochain/holochain-rust/pull/1877)
- Fixed infinite gossip loop due to non-deterministic creation of virtual chain header headers [1904](https://github.com/holochain/holochain-rust/pull/1904)
### Security

## [0.0.38-alpha14] - 2019-11-13

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.38-alpha13] - 2019-11-12

### Added

### Changed

### Deprecated

### Removed

- Removes the `__META__` fields in a .dna.json that allowed it to be unpacked. Removes the `hc unpackage` CLI option. [#1864](https://github.com/holochain/holochain-rust/pull/1864)

### Fixed

### Security

## [0.0.38-alpha12] - 2019-11-11

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.38-alpha9] - 2019-11-11

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.38-alpha8] - 2019-11-11

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.38-alpha7] - 2019-11-11

### Added
* Adds try-o-rama remote server provisioning via trycp [#1780](https://github.com/holochain/holochain-rust/pull/1780)
  This also adds nix-shell commands:
  - `hc-trycp-server-install` which installs the trycp-server
  - `hc-trycp-server` which runs the trycp-server
* Adds instrumentation to measure and publish. performance. Introduces `hc-metrics` command to parse logs and generate statistics. [#1810](https://github.com/holochain/holochain-rust/pull/1810)

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.38-alpha6] - 2019-11-10

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.38-alpha5] - 2019-11-10

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.38-alpha4] - 2019-11-10

### Added
* Adds instrumentation to measure and publish. performance. Introduces `hc-metrics` command to parse logs and generate statistics. [#1810](https://github.com/holochain/holochain-rust/pull/1810)
### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.38-alpha2] - 2019-11-08

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.38-alpha1] - 2019-11-08

### Added
* The sim2h switch-board server is now caching if a node is missing data and periodically checks back in. This makes it more resilient against unforseen problems like connection drops which otherwise could only be recovered through an explicit reconnection of the node. [#1834](https://github.com/holochain/holochain-rust/pull/1834)

### Changed
* DNA is now checked for invalid zome artifacts. Validation callbacks that fail unexpectedly will now panic rather than fail validation. `hc package` `--strip-meta` flag is now `--include-meta`. [#1838](https://github.com/holochain/holochain-rust/pull/1838)

### Deprecated

### Removed

### Fixed

* Loading of instances from storage was broken and ended up in partially loaded states. This got fixed with [#1836](https://github.com/holochain/holochain-rust/pull/1836).

### Security

## [0.0.37-alpha12] - 2019-11-06

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.37-alpha11] - 2019-11-06

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.37-alpha10] - 2019-11-06

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.37-alpha9] - 2019-11-06

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.37-alpha8] - 2019-11-06

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.37-alpha7] - 2019-11-06

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.37-alpha6] - 2019-11-05

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.37-alpha5] - 2019-11-05

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.37-alpha4] - 2019-11-05

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.37-alpha3] - 2019-11-05

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.37-alpha2] - 2019-11-05

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.37-alpha1] - 2019-11-05

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.36-alpha1] - 2019-11-04

### Added

*  Allows the HC CLI to generate zomes from template repos. This will by default use the default holochain template repos (holochain/rust-zome-template and holochain/rust-proc-zome-template) but can also point to custom templates in remote repos or locally (e.g. `hc generate zomes/my_zome https://github.com/org/some-custom-zome-template`). [#1565](https://github.com/holochain/holochain-rust/pull/1565)
* Adds option `--property` to `hc hash` that sets DNA properties for hash calculation. [#1807](https://github.com/holochain/holochain-rust/pull/1807)
* Adds a prelude module to the HDK. Adding the statement `use hdk::prelude::*` should be enough for 90% of zome development [#1816](https://github.com/holochain/holochain-rust/pull/1816)
* Adds a special DNA property sim2h_url that, if set, overrides the conductor wide setting for the network configuration variable sim2h_url. [PR#1828](https://github.com/holochain/holochain-rust/pull/1828)
### Changed

### Deprecated

### Removed

### Fixed

* Fixes handling if DNA properties during `hc package`. DNA properties mentioned in the DNA's JSON manifest are now included in the package. [PR#1828](https://github.com/holochain/holochain-rust/pull/1828)

### Security

## [0.0.35-alpha7] - 2019-10-30

### Added

*  Allows the HC CLI to generate zomes from template repos. This will by default use the default holochain template repos (holochain/rust-zome-template and holochain/rust-proc-zome-template) but can also point to custom templates in remote repos or locally (e.g. `hc generate zomes/my_zome https://github.com/org/some-custom-zome-template`). [#1565](https://github.com/holochain/holochain-rust/pull/1565)
* Adds option `--property` to `hc hash` that sets DNA properties for hash calculation. [#1807](https://github.com/holochain/holochain-rust/pull/1807)
### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.34-alpha1] - 2019-10-25

### Added

*  Adds the holochain_persistence_lmdb crate and makes this an option for the instance config. This is now the default store implementation. [#1758](https://github.com/holochain/holochain-rust/pull/1758)

### Changed

* Custom signals that are emitted from DNA/zome code ("user" signals) are now send to all admin interfaces to enable UI switching logic in Holoscape [#1799](https://github.com/holochain/holochain-rust/pull/1799)

### Deprecated

### Removed

### Fixed

### Security

## [0.0.33-alpha6] - 2019-10-24

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.33-alpha5] - 2019-10-23

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.33-alpha4] - 2019-10-23

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.33-alpha3] - 2019-10-23

### Added
* Adds a network back-end: `sim2h` and all corresponding integration. [#1744](https://github.com/holochain/holochain-rust/pull/1744)

  [Sim2h](https://github.com/holochain/sim2h) is the next iteration of sim1h.
  In contrast to sim1h, it does not use a centralized database but a
  centralized in-memory network that connects Holochain instances
  like a switch-board.

  It is much faster than sim1h and will be able to implement Holochain
  membranes based on the agent IDs and the `validate_agent` callback.

  It can be used by configuring conductors like so:
  ```toml
  [network]
  type = "sim2h"
  sim2h_url = "wss://localhost:9000"
  ```
  with `sim2h_url` pointing to a running `sim2h_server` instance.

  This also adds nix-shell commands:
  - `hc-sim2h-server-install` which installs the sim2h-server
  - `hc-sim2h-server-uninstall` which removes the sim2h-server
  - `hc-sim2h-server` which starts the server with on
    port 9000 (can be changed with `-p`) and with  debug logs enabled
  - `hc-app-spec-test-sim2h` which runs the integration tests with
    networking configured to sim2h (expects to find a running
    sim2h_server on localhost:9000)
### Changed

### Deprecated

### Removed

### Fixed

- Fixed the frequent deadlocks that would occur on conductor shutdown [#1752](https://github.com/holochain/holochain-rust/pull/1752)

### Security

## [0.0.32-alpha2] - 2019-10-08

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.32-alpha1] - 2019-10-08

### Added

*  Adds the `--properties`/`-p` flag to `hc package` which takes a stringifed JSON object to be inserted in the .dna.json under the properties field. This will alter the DNA hash and can therefore be used for fork DNAs from their source code. [#1720](https://github.com/holochain/holochain-rust/pull/1720)
* Adds publishing of headers again after rollback. Header publishing is now its own action rather than part of the `Publish` action that plays nicely with the testing framework. It also adds header entries to the author list so they are gossiped properly. [#1640](https://github.com/holochain/holochain-rust/pull/1640).
* Adds some deadlock diagnostic tools to detect when any mutex has been locked for a long time, and prints the backtrace of the moment it was locked [#1743](https://github.com/holochain/holochain-rust/pull/1743)

### Changed

* Updates to work with version 0.0.13 of lib3h  [#1737](https://github.com/holochain/holochain-rust/pull/1737)

### Deprecated

### Removed

### Fixed

### Security

## [0.0.31-alpha1] - 2019-10-03

### Added

* Adds publishing of headers again after rollback. Header publishing is now its own action rather than part of the `Publish` action that plays nicely with the testing framework. It also adds header entries to the author list so they are gossiped properly. [#1640](https://github.com/holochain/holochain-rust/pull/1640).

* Adds new networking back-end `sim1h` which can be configured in conductor config with:
    ```toml
    [network]
    type = "sim1h"
    dynamo_url = "http://localhost:8000"
    ```
    [#1725](https://github.com/holochain/holochain-rust/pull/1725)
* Adds nix-shell commands for running app-spec tests with different network implementations
  - `hc-app-spec-test-sim1h`
  - `hc-app-spec-test-n3h`
  - `hc-app-spec-test-memory`

  [#1725](https://github.com/holochain/holochain-rust/pull/1725)

* Adds nix-shell commands for running a local DynamoDB instance:
  - `dynamodb` and
  - `dynamodb-memory`

  [#1725](https://github.com/holochain/holochain-rust/pull/1725)

* Adds zome+function name to ConsistencyEvent::Hold representation for pending zome function call returns for better hachiko timeouts. [#1725](https://github.com/holochain/holochain-rust/pull/1725)

* Adds `UUID` to DNA configs which will change the DNA when initializing an instance with it and sets the given UUID. This disables the hash check of the DNA if set. [#1724](https://github.com/holochain/holochain-rust/pull/1724) [#1725](https://github.com/holochain/holochain-rust/pull/1725)

### Changed
* Converts app-spec tests to the new multi-conductor [try-o-rama](https://github.com/holochain/try-o-rama) [#1725](https://github.com/holochain/holochain-rust/pull/1725)

### Deprecated

### Removed

### Fixed
* Fixes several conditions that lead to occasional deadlocks [#1725](https://github.com/holochain/holochain-rust/pull/1725)


### Security

## [0.0.30-alpha6] - 2019-09-17

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.30-alpha5] - 2019-09-16

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.30-alpha4] - 2019-09-16

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.30-alpha23] - 2019-09-16

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.30-alpha2] - 2019-09-15

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.30-alpha1] - 2019-09-15

### Added

* Adds EncryptedSeed and seed.encrypt() allow for easy passphrase encrypting/decrypting of any of the existing seed types. Adds the MnemonicableSeed trait allows seeds to be converted to/from BIP39 mnemonics. [#1687](https://github.com/holochain/holochain-rust/pull/1687)
* added nix for `hc-conductor-install` and `hc-conductor-uninstall` based on `cargo` [#1689](https://github.com/holochain/holochain-rust/pull/1689)
* When loading a hand-written or generated conductor config containing a TestAgent (`test_agent = true`), rewrite the config file so that the test agent's `public_address` is correct, rather than the arbitrary value that was specified before the `public_address` was actually known. [#1692](https://github.com/holochain/holochain-rust/pull/1692)

### Changed

* ConsistencySignal "events" are now serialized to strings before being emitted. [#1691](https://github.com/holochain/holochain-rust/pull/1691)

### Deprecated

### Removed

### Fixed

### Security

## [0.0.29-alpha2] - 2019-08-26

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.29-alpha1] - 2019-08-26

### Added

* If there is an HDK mismatch in the zome, a warning is thrown.Also gives ability to get current HDK version in zomes[#1658](https://github.com/holochain/holochain-rust/pull/1658)
* Conductor API debug functions added:
    * `debug/running_instances`: returns array of running instance IDs
    * `debug/state_dump`: returns a state dump for a given instance
    * `debug/fetch_cas`: returns the content for a given entry address and instance ID

  Also added the source to the state dump.
  [#1661](https://github.com/holochain/holochain-rust/pull/1661)

* Add `alias` to instance references in interfaces to decouple hard-coded instance references in hApp UIs from conductor configs. [#1676](https://github.com/holochain/holochain-rust/pull/1676)
### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.28-alpha1] - 2019-08-18

### Added
* Ability to provide passphrase to lock/unlock keystores via IPC unix domain socket added. [#1646](https://github.com/holochain/holochain-rust/pull/1646)

* Documentation for our links ecosystem [#1628](https://github.com/holochain/holochain-rust/pull/1628)
### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.27-alpha1] - 2019-08-08

### Added

* New logging implementation added as a subcrate : a fast logger with a filtering capability using regex expressions, please so [logging](logging) for more details. [#1537](https://github.com/holochain/holochain-rust/pull/1537) and [#1639](https://github.com/holochain/holochain-rust/pull/1639)

### Changed

- Bump dependent crate versions (holochain_persistence 0.0.7, holochain_serialization 0.0.7, lib3h 0.0.10) in preparation futures 0.3.0-alpha17 which will allow us to shift to the upcoming Rust 1.38.0 beta [#1632](https://github.com/holochain/holochain-rust/pull/1632)

### Deprecated

### Removed

### Fixed

### Security

## [0.0.26-alpha1] - 2019-08-05

### Added

### Changed
- State dump debugging: A new config flag got added that activates dumping of core's redux state every ten seconds in a human readable form: [#1601](https://github.com/holochain/holochain-rust/pull/1601)
- The static file server has been replaced and now uses the Nickel crate intead of Hyper. It now correctly sets content type headers and can be configured to bind to a different address in the conductor config toml [#1595](https://github.com/holochain/holochain-rust/pull/1595)
- Optimized get_links so that fewer network calls are made overrall [#1607](https://github.com/holochain/holochain-rust/pull/1607)

- DEPRECATION WARNING, conductor static UI server is to be removed in an upcoming release. Devs will receive a warning when starting a conductor with a UI server configured [PR#1602](https://github.com/holochain/holochain-rust/pull/1602)

### Deprecated

### Removed

### Fixed
- When using agent config with `test_agent = true`, the conductor was checking the `public_address` field against the generated keystore. No longer so. [PR#1629](https://github.com/holochain/holochain-rust/pull/1629)

### Security

## [0.0.25-alpha1] - 2019-07-26

### Added

### Changed
- **Breaking Change** genesis function now renamed to init [#1508](https://github.com/holochain/holochain-rust/pull/1508)
- **BREAKING:** Zomes must now include a `validate_agent` callback. If this rejects in any zome the DNA will not start. This can be used to enforce membrane requirements. [#1497](https://github.com/holochain/holochain-rust/pull/1497)
- Added a `get_links_count` method which allows the user to get number of links by base and tag [#1568](https://github.com/holochain/holochain-rust/pull/1568)### Changed
- The Conductor will shut down gracefully when receiving SIGINT (i.e. Ctrl+C) or SIGKILL, also causing a graceful shutdown of an attached n3h instance, if running [#1599](https://github.com/holochain/holochain-rust/pull/1599)

### Deprecated

### Removed

### Fixed
- Fixed problem with `hc run` that was introduced by [Conductor config sanitizing](https://github.com/holochain/holochain-rust/pull/1335) a week ago: The conductor config now needs to include the correct hash of each configured DNA file. [#1603](https://github.com/holochain/holochain-rust/pull/1603) adds the proper hash to the internally created conductor config that `hc run` runs.

### Security

## [0.0.24-alpha2] - 2019-07-15

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.24-alpha1] - 2019-07-15

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.23-alpha1] - 2019-07-11

### Added
- Discrepancy between DNA hashes are now checked and reported to the user through logs [#1335](https://github.com/holochain/holochain-rust/pull/1335).

### Changed

- *Breaking Change* Validation callback now shows consistent behavior when called on the authoring node during entry commit time, and when called by validating nodes being requested to hold the entry.  In both cases the a FullChain validation package now does NOT include the about-to-be-added entry.  Some validation functions were relying on the behavior of having the entry be at the top of the chain in the Hold case, and using the EntryLifecycle flag value to distinguish the two cases.   Please note that in the future this flag may be going away! [#1563](https://github.com/holochain/holochain-rust/pull/1563)
- *Breaking Change* Format of `.hcbuild` files that are run by `hc` changed: `steps` is now an array so we have deterministic ordering of build steps. - In order to apply WASM size optimizations to our app-spec test suite, we had to make more sophisticated use of the `.hcbuild` files with a sequence of consecutive steps. The former implementation with a map had to changed to an array. [#1577](https://github.com/holochain/holochain-rust/pull/1577)
### Deprecated

- The EntryLifecycle flags in validation may be going away.  If you have a use-case that requires this, please tell us.

### Removed

### Fixed

### Security

## [0.0.22-alpha1] - 2019-07-04

### Added
- Added `properties` to entry definitions (not to the entries themselved). These can be retrieved using the `entry_type_properties` HDK function [#1337](https://github.com/holochain/holochain-rust/pull/1337)
- *Breaking Change* Added type field to conductor network configuration.  You must add `type="n3h"` for current config files to work.  [#1540](https://github.com/holochain/holochain-rust/pull/1540)
- Added `Encryption` and `Decryption` methods in the HDK [#1534](https://github.com/holochain/holochain-rust/pull/1534)
- Adds `hc hash` CLI subcommand. Can be used to compute the hash of the DNA in the current dist directory or passed a path to a DNA with the --path flag [#1562](https://github.com/holochain/holochain-rust/pull/1562)
- Adds a --dna flag to the CLI so `hc run` can run DNAs outside the standard ./dist/ directory [1561](https://github.com/holochain/holochain-rust/pull/1561)

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.21-alpha1] - 2019-06-26

### Added
- Added `Crud Status` information to link data in get_links as well as query through `LinkStatusRequest` [#1337](https://github.com/holochain/holochain-rust/pull/1337)
- The `hc` tool can now generate template zomes that use the new proc macro HDK [#1511](https://github.com/holochain/holochain-rust/pull/1511)
- Added a MVP implementation of [Signals](https://github.com/holochain/holochain-rust/blob/develop/doc/architecture/decisions/0013-signals-listeners-model-and-api.md) that introduces `hdk::emit_signal(name, payload)` [#1516](https://github.com/holochain/holochain-rust/pull/1516)

### Changed
- The barebones tests produced by `hc init` now use the Diorama testing framework rather than holochain-nodejs [#1532](https://github.com/holochain/holochain-rust/pull/1532)

- **Breaking change** - `holochain_core_types_derive` and `holochain_core_types` are split into `holochain_json_derive`, `holochain_json_api`, `holochain_persistence_api` [#1505](https://github.com/holochain/holochain-rust/pull/1505)

- Fix dangling references of `core_types_derive` and related imports [#1551](https://github.com/holochain/holochain-rust/pull/1551)

### Deprecated

### Removed

### Fixed

### Security

## [0.0.20-alpha3] - 2019-06-17

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.20-alpha2] - 2019-06-17

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.20-alpha1] - 2019-06-16

### Added

- **Breaking change** - renames `emit_trace_signals` to `signals.trace` in conductor config [#1431](https://github.com/holochain/holochain-rust/pull/1431)
- "Consistency" signals added, which aid determinism in end-to-end tests, configurable through `signals.consistency` conductor config [#1431](https://github.com/holochain/holochain-rust/pull/1431)
- Uses regex matching for `get_links` tags and type. Probably not a breaking change but be careful of subset matching (e.g. `some` will match against `some-tag` but `^some$` will not.) [#1453](https://github.com/holochain/holochain-rust/pull/1453)
- `Tombstone` functionality added on eaviquery, this makes sure that the delete links is not determined by order but determined by a `tombstone set` which takes precedence over everything. [#1363](https://github.com/holochain/holochain-rust/pull/1363)

### Deprecated

### Removed

- **Breaking change** - migrates nodejs_conductor and nodejs_waiter to holochain-nodejs repo [#1510](https://github.com/holochain/holochain-rust/pull/1510)

### Fixed

### Security

## [0.0.19-alpha1] - 2019-06-10

### Added
- Error log output added for errors occurring during `hdk::call`, including bridge call errors [#1448](https://github.com/holochain/holochain-rust/pull/1448).
- New `uuid` parameter for `admin/dna/install_from_file`, to set the UUID of the installed DNA, changing its hash [#1425](https://github.com/holochain/holochain-rust/pull/1425)
- **BREAKING:** Conductor configuration checks for bridges added [#1461](https://github.com/holochain/holochain-rust/pull/1461). Conductor will bail with an error message if the configuration of bridges between instances does not match the bridge requirements defined in the caller instance's DNA (required bridge missing, DNA hash mismatch, trait mismatch) or if a bridge with the handle specified in the config can not be found in the caller's DNA.

### Changed
- Added a Vagrant file to support nix-shell compatible VMs on windows etc. [#1433](https://github.com/holochain/holochain-rust/pull/1433)
- Adds TryInto implementation from JsonString to generic result types. This makes bridge calls much easier to implement safely [#1464](https://github.com/holochain/holochain-rust/pull/1464)
- Changes the responses when using `hdk::call` to call across a bridge to make it consistent with calling between zomes  [#1487](https://github.com/holochain/holochain-rust/pull/1487)

### Changed

### Deprecated

### Removed

### Fixed

- Adding bridges dynamically via an admin interface works now without rebooting the conductor. [#1476](https://github.com/holochain/holochain-rust/pull/1476)
- `hdk::query` results are filtered now to not contain DNA entries since they can easily be several MBs of size which breaks our current limitation of 640k of WASM memory. [#1490](https://github.com/holochain/holochain-rust/pull/1490)

### Security

## [0.0.18-alpha1] - 2019-06-03

### Added

### Changed

- **Breaking change** - renames `emit_trace_signals` to `signals.trace` in conductor config [#1431](https://github.com/holochain/holochain-rust/pull/1431)
- "Consistency" signals added, which aid determinism in end-to-end tests, configurable through `signals.consistency` conductor config [#1431](https://github.com/holochain/holochain-rust/pull/1431)

### Deprecated

### Removed

### Fixed

### Security

## [0.0.17-alpha2] - 2019-05-27

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.17-alpha1] - 2019-05-27

### Added

- Option to show NPM output when pulling deps during `hc test` [PR#1401](https://github.com/holochain/holochain-rust/pull/1401)
- Adds scaffolding/skeleton for a future WASM conductor [#894](https://github.com/holochain/holochain-rust/pull/894)

### Changed

- **Breaking Change** Renames link tags to link_type. Adds new link tag which can be any string. This is available in validation of links and links can be retrieved based on their tag+type, just tag, just type or retrieve all. `hdk::link_entries` and `hdk::get_links` now required an extra parameter.  [#1402](https://github.com/holochain/holochain-rust/pull/1402).
- Option to show NPM output when pulling deps during `hc test` [PR#1401](https://github.com/holochain/holochain-rust/pull/1401)
- Adds scaffolding/skeleton for a future WASM conductor [#894](https://github.com/holochain/holochain-rust/pull/894)
- Adds PROPERTIES static to the HDK which contains a JsonString with the DNA properties object. Also adds a body to the `hdk::properties` stub which allows retrieving fields from the properties object as JsonString. [#1418](https://github.com/holochain/holochain-rust/pull/1418)
- Conductor now persists its config in the config root (e.g. `home/peter/.config/holochain/conductor` rather than `~/.holochain`) [#1386](https://github.com/holochain/holochain-rust/pull/1386)
- Default N3H mode as set when spawned by the conductor got set to "REAL". [#1282](https://github.com/holochain/holochain-rust/pull/1282)
- Internal signals renamed to Trace signals, with ability to opt in or out through `emit_trace_signals` conductor config [#1428](https://github.com/holochain/holochain-rust/pull/1428)

### Deprecated

### Removed

### Fixed

### Security

## [0.0.16-alpha1] - 2019-05-16

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.15-alpha1] - 2019-05-09

### Added

- Adds new RPC method to conductor `test/agent/add` which adds an agent but does not save the config or generate a keystore. This is added to enable tests that run against the Rust conductor [PR#1359](https://github.com/holochain/holochain-rust/pull/1359)
- Adds `from` argument to the `receive` callback. [#1382](https://github.com/holochain/holochain-rust/pull/1382)
- Adds a new hdk::keystore_get_public_key function which returns the public key of a key secret from the keystore.  [#1383](https://github.com/holochain/holochain-rust/pull/1383)
- Adds hdk::commit_capability_grant() for zome functions to be able to create [capability grant](doc/architecture/decisions/0017-capabilities.md)  [#1285](https://github.com/holochain/holochain-rust/pull/1285)
- Adds hdk::commit_entry_result() which features: optional argument to include additional provenances. [#1320](https://github.com/holochain/holochain-rust/pull/1320)

### Changed

- Updated linked [n3h](https://github.com/holochain/n3h) version to v0.0.12-alpha [#1369](https://github.com/holochain/holochain-rust/pull/1369)
- pin mozilla overlay to latest commit in nix [#1375](https://github.com/holochain/holochain-rust/pull/1375)

### Deprecated

### Removed

### Fixed

### Security

## [0.0.15-alpha1] - 2019-05-02

### Added

- Adds hdk::commit_entry_result() which features: optional argument to include additional provenances. [#1320](https://github.com/holochain/holochain-rust/pull/1320)
- default.nix file added to facilitate `nix-env` based binary installation [#1356](https://github.com/holochain/holochain-rust/pull/1356)

### Changed
- Changes `LinkAdd` and `RemoveEntry` so that they return a hash instead of a null [#1343](https://github.com/holochain/holochain-rust/pull/1343)
- Merged `default.nix` and `shell.nix` to improve `nix-shell` handling [#1371](https://github.com/holochain/holochain-rust/pull/1371)

### Deprecated

### Removed

### Fixed

### Security

## [0.0.13-alpha1] - 2019-04-29

### Added
- Adds hdk::grant_capability() for zome functions to be able to create [capability grant](doc/architecture/decisions/0017-capabilities.md)  [#1285](https://github.com/holochain/holochain-rust/pull/1285)
- `nix-shell` includes latest `hc` and `holochain` binaries [#1306](https://github.com/holochain/holochain-rust/pull/1306)
- Adds `hc-cli-uninstall` and `hc-conductor-rust-uninstall` to drop local development installations of these binaries that would override dist binaries [#1351](https://github.com/holochain/holochain-rust/pull/1351)

### Changed
- changed JSON-RPC Zome call `params` key to `args` for clarity (due to confusion between JSON-RPC `params` and Holochain `params` keys): see [#1203](https://github.com/holochain/holochain-rust/pull/1203) and [#1271](https://github.com/holochain/holochain-rust/pull/1271)
- Remove sleeps during network initialization, block until P2pReady event is received [#1284](https://github.com/holochain/holochain-rust/pull/1284).
- refactored `shell.nix` into `holonix` directory for rationalized `nix-shell` commands and easier maintenance and clarity. [#1292](https://github.com/holochain/holochain-rust/pull/1292)
  - note: `hc-test` is now `hc-rust-test` and `hc-test-all` is now `hc-test`

### Deprecated
- `params` Zome call argument deprecated in favor of `args`. [#1271](https://github.com/holochain/holochain-rust/pull/1271)

### Removed

### Fixed
- Windows-only: Spawned `node.exe` process used by network module now closes properly on holochain termination [#1293](https://github.com/holochain/holochain-rust/pull/1293)

### Security

## [0.0.12-alpha1] - 2019-04-21

### Added
- Allows the user to get headers using GetLinkOptions. [#1250](https://github.com/holochain/holochain-rust/pull/1250)

- `Config.bridge` added to Scenario API, allowing bridges to be configured [#1259]()https://github.com/holochain/holochain-rust/pull/1259

- Adds CAPABILITY_REQ global for access from a zome function call to the capability request that was used to make the call. This is important for doing validation of provenance for a zome call that wants to create a [capability grant](doc/architecture/decisions/0017-capabilities.md). [#1273](https://github.com/holochain/holochain-rust/pull/1273)

### Changed

- Increased timeout on n3h spawn and wait for `#P2P-READY#` message [#1276](https://github.com/holochain/holochain-rust/pull/1276).
- Clarifies the error received when attempting to add a DNA whose expected hash mismatches the actual hash [#1287](https://github.com/holochain/holochain-rust/pull/1287).
- Binary tarballs no longer extract to a subdirectory [#1265](https://github.com/holochain/holochain-rust/pull/1265)
- Linux binary tarballs are now named `generic` rather than `ubuntu` [#1265](https://github.com/holochain/holochain-rust/pull/1265)
- When getting links, the result has changed from `addresses: Vec<Address>` to `links: Vec<LinksResult>`. [#1250](https://github.com/holochain/holochain-rust/pull/1250)

### Deprecated

### Removed

### Fixed

- Windows-only: Spawned `node.exe` process used by network module now closes properly on holochain termination [#1293](https://github.com/holochain/holochain-rust/pull/1293)

- Don't publish private zome entries [#1233](https://github.com/holochain/holochain-rust/pull/1233)

- Fix unspecified errors that can occur during entry deletion/update [#1266](https://github.com/holochain/holochain-rust/pull/1266)

### Security

## [0.0.11-alpha1] - 2019-04-11

### Added

### Changed

- Performance optimization: Don't clone and parse WASM binaries for each distinct WASM execution such as Zome function calls and validation callbacks. Instead hold only one parsed module instance per zome on the heap and use that to initialize WASM instances. [#1211](https://github.com/holochain/holochain-rust/pull/1211)
- OpenSSL is vendored (statically linked) on nixos and other linux [#1245](https://github.com/holochain/holochain-rust/pull/1245)

### Deprecated

### Removed

### Fixed

- Fixes problem where Scenario tests hang when throwing an error during `runTape` [#1232](https://github.com/holochain/holochain-rust/pull/1232)

### Security

## [0.0.10-alpha2] - 2019-04-04

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.10-alpha1] - 2019-04-04

### Added

- Adds conductor handling of agent key creation in the context of DPKI [#1182](https://github.com/holochain/holochain-rust/pull/1182)
- Adds a `--path` option to `hc keygen` to specify the location of the generated keybundle. [#1194](https://github.com/holochain/holochain-rust/pull/1194)
- Adds pickle db for cas and eav [#1178](https://github.com/holochain/holochain-rust/pull/1178)
- Adds a `--quiet` option to `hc keygen` for machine-readable output, intended for use in scripts. [#1197](https://github.com/holochain/holochain-rust/pull/1197)
- Adds logging output for every failed WASM execution showing the call that caused this error. [#1200](https://github.com/holochain/holochain-rust/pull/1200) This helps with debugging "Arguement Deserialization failed" errors.
- Adds DNA hash to `hc package` output [#1212](https://github.com/holochain/holochain-rust/pull/1212)

### Changed

- `add_agent()` admin function now creates keystore file instead of just recording file in config [#1182](https://github.com/holochain/holochain-rust/pull/1182)
- One-time-signing now takes a vector of payloads, and returns a vector of signatures. [#1193](https://github.com/holochain/holochain-rust/pull/1193)
- Pins nixpkgs to Holo-Host channel in shell and CI [#1162](https://github.com/holochain/holochain-rust/pull/1162)

### Deprecated

### Removed

- Removes deprecated zome calling route [#1147](https://github.com/holochain/holochain-rust/pull/1147). This is a breaking change for users of hc-web-client prior to version 0.1.3.  Please upgrade to 0.1.3 or later and use the callZome syntax.
- Removes JsonString::From<String> and replaces it with JsonString::from_json(&str). This makes conversions more explicit and allows for validating that the string is actually valid json [#1184](https://github.com/holochain/holochain-rust/pull/1184)

### Fixed

-This pull request fixes the various issues with the pickledb implementation. Better guards and directory fixes [#1202]
(https://github.com/holochain/holochain-rust/pull/1202)

### Security

## [0.0.9-alpha] - 2019-03-31

### Added
- Adds hdk access to keystore [#1148](https://github.com/holochain/holochain-rust/pull/1148)

### Changed
- Performance optimization: don't recalculate DNA hash during handling of every network message but instead cache the DNA hash. [PR#1163](https://github.com/holochain/holochain-rust/pull/1163)

### Deprecated

### Removed

### Fixed

### Security


## [0.0.8-alpha] - 2019-03-21

### Added

- Adds Validation For CrudStatus as well as changes api for Crud and Link Validation Rules. [PR#1117] (https://github.com/holochain/holochain-rust/pull/1117)
- Adds `nix-shell` support for Mac OS X [#1132](https://github.com/holochain/holochain-rust/pull/1132)
- Adds `hc-test-all` command to `nix-shell` [#1132](https://github.com/holochain/holochain-rust/pull/1132)
- Adds `./scripts/nix/pod.sh` script to isolate/debug `nix-shell` commands [#1139](https://github.com/holochain/holochain-rust/pull/1139)
- Adds getting of Headers over the network [#1141](https://github.com/holochain/holochain-rust/pull/1141)
- Adds keystore and passphrase management service [#1104](https://github.com/holochain/holochain-rust/pull/1104)
- Adds tooling to manage dependencies in Cargo.toml [#1140](https://github.com/holochain/holochain-rust/pull/1140)
- Adds ability to enable logging via flag (`--logging`) to `hc run` command [#1151](https://github.com/holochain/holochain-rust/pull/1151)
- Adds `hc chain` command, which prints a raw text dump of a source chain [#1126](https://github.com/holochain/holochain-rust/pull/1126)


### Changed
- Conductor now waits for N3H to return p2p bindings [#1149](https://github.com/holochain/holochain-rust/pull/1149)
- `nix-shell` is now the recommended development approach on supported platforms [#1132](https://github.com/holochain/holochain-rust/pull/1132)
- Pins every dependant crate version with `=x.y.z` at the Cargo.toml level [#1140](https://github.com/holochain/holochain-rust/pull/1140)
- Breaking Change: `key_file` value now renamed to `keystore_file` in both config.toml files and the conductor's `admin/agent/add` interface [#1104](https://github.com/holochain/holochain-rust/pull/1104)
- EAVI adds are now optimized [#1166](https://github.com/holochain/holochain-rust/pull/1166)

### Deprecated

### Removed

- Removes all Cargo.lock files [#1140](https://github.com/holochain/holochain-rust/pull/1140)

### Fixed
- Adds Validation for Crud Reinstates EntryLifecycle. [PR#1143] (https://github.com/holochain/holochain-rust/pull/1143)
### Security

## [0.0.7-alpha] - 2019-03-19

### Added

- Adds the ability to pass in the token and provenance in zome calls for generating the capability request for the call. [PR#1077](https://github.com/holochain/holochain-rust/pull/1077)

### Changed

- Instantiate instance when creating through admin interface [#1067](https://github.com/holochain/holochain-rust/pull/1067)
- Use Content-type: application/json for remote signing service HTTP requests [#1067](https://github.com/holochain/holochain-rust/pull/1067)
- Check for duplicate IDs during integrity check [#1067](https://github.com/holochain/holochain-rust/pull/1067)

### Deprecated

### Removed

### Fixed
- Conductors running on Windows will be able to hit '/' route for UI server [PR#1128](https://github.com/holochain/holochain-rust/pull/1128)

### Security


## [0.0.6-alpha] - 2019-03-11

### Changed
- Replaces libzmq (zeromq) with websockets for ipc communication to networking module [#1055](https://github.com/holochain/holochain-rust/pull/1055)
- Changes `apt-get` dependencies installed for libsodium across linux OS [#1105](https://github.com/holochain/holochain-rust/pull/1105)

### Removed
- Removes bespoke `rust_sodium-sys` crate (using upstream now) [#1105](https://github.com/holochain/holochain-rust/pull/1105)

### Added
- New network setting via environment variable HC_N3H_LOG_LEVEL [#1085](https://github.com/holochain/holochain-rust/pull/1085)
- Ability to sign data via `hdk::sign` using the agent key [PR#1080](https://github.com/holochain/holochain-rust/pull/1080)
- Adds PUBLIC_TOKEN global variable for use in hdk::call in calling public functions. [PR#895](https://github.com/holochain/holochain-rust/pull/895)
- Adds an [ADR](doc/architecture/decisions/0017-capabilities.md) for capabilities [#895](https://github.com/holochain/holochain-rust/pull/895)
- CrudStatus works over network [#1048](https://github.com/holochain/holochain-rust/pull/1048)
- Adds utils submodule of hdk which contains the following helper functions [#1006](https://github.com/holochain/holochain-rust/pull/10006):
  - get_links_and_load_type - calls try_from for a given type when getting links
  - get_as_type - Similar but for a single entry
  - link_entries_bidir - Same as link_entries but creates link in both directions
  - commit_and_link - Save a line and commit and link in a single function
- Adds a `call` route to the json rpc for the conductor for making zome calls [PR#1090](https://github.com/holochain/holochain-rust/pull/1090).  Please note this route deprecates the `instance_id/zome/function` which will be removed in the future
- The `admin/dna/install_from_file` RPC method now takes an optional `expected_hash`, which performs an integrity check of the DNA file before installing it in the conductor [PR#1093](https://github.com/holochain/holochain-rust/pull/1093)
- Adds empty API function definitions to HDK that are only compiled for test targets to enable Rust native unit tests for Zomes [#989](https://github.com/holochain/holochain-rust/pull/989)
- Moves Crud Status tests to app_spec [#1096](https://github.com/holochain/holochain-rust/pull/1096)
- Adds cold build tests + support for debian and ubuntu xenial [#1105](https://github.com/holochain/holochain-rust/pull/1105)

### Fixed
- Validation of link entries gets retried now if base or target of the link were not yet accessible on the validating node. This fixes a bug where links have been invalid due to network timing issues [PR#1054](https://github.com/holochain/holochain-rust/pull/1054)
- Validation of any entry gets retried now if the validation package could not be retrieved from the source [PR#1059](https://github.com/holochain/holochain-rust/pull/1059)
- Scenario tests are more lenient to SyntaxError, TypeError, and other JS errors: buggy tests now merely fail rather than hanging indefinitely [#1091](https://github.com/holochain/holochain-rust/pull/1091)
- Fixes docker builds for `holochain/holochain-rust:develop` [#1107](https://github.com/holochain/holochain-rust/pull/1107)

## [0.0.5-alpha] - 2019-03-01

### Changed
- Relaxes Node JS version to 8.x in nix-shell [PR#955](https://github.com/holochain/holochain-rust/pull/955)
- Updates develop docker tag to use nix [PR#955](https://github.com/holochain/holochain-rust/pull/955)
- Updates bash script shebang to be nixos friendly [PR#955](https://github.com/holochain/holochain-rust/pull/955)
- Changes file name for cli packaging [PR#1036](https://github.com/holochain/holochain-rust/pull/1036)
  - `bundle.json` & `.hcpkg` unified to `YOUR_DNA_NAME.dna.json`
  - `.build` files renamed to `.hcbuild`
  - `hc package` now builds to `dist` directory by default, to match how `hc test` works

### Removed
- Removes legacy docker files [PR#955](https://github.com/holochain/holochain-rust/pull/955)

### Added
- Adds a panic handler to HDK-Rust and that reroutes infos about panics happening inside the WASM Ribosome to the instances logger [PR#1029](https://github.com/holochain/holochain-rust/pull/1029)
- Adds cmake and qt to mac os x install script [PR#955](https://github.com/holochain/holochain-rust/pull/955)
- Adds the current git-commit hash to the compile code of the core, and checks (with warning) for the same hash that was used to compile the wasm [PR#1050](https://github.com/holochain/holochain-rust/pull/1036)

## [0.0.4-alpha] - 2019-02-15

### Fixed
- Futures handling and zome function execution refactored which enables using complex API functions like `commit_entry` in callbacks such as `receive`.  This also fixes long standing flaky tests and blocking behaviors we have been experiencing. [#991](https://github.com/holochain/holochain-rust/pull/991)
### Changed
- Capabilities now separated from function declarations and renamed to `traits` in `define_zome!` and calling zome functions no longer uses capability name parameter [#997](https://github.com/holochain/holochain-rust/pull/997) & [#791](https://github.com/holochain/holochain-rust/pull/791)
- `hash` properties for `UiBundleConfiguration` and `DnaConfiguration` in Conductor config files is now optional [#966](https://github.com/holochain/holochain-rust/pull/966)
- `ChainHeader::sources()` is now `ChainHeader::provenances()` which stores both source address, and signature  [#932](https://github.com/holochain/holochain-rust/pull/932)
- `hdk::get_entry_results` supports return of ChainHeaders for all agents who have committed the same entry [#932](https://github.com/holochain/holochain-rust/pull/932)
- Renames the term Container and all references to it to Conductor [#942](https://github.com/holochain/holochain-rust/pull/942)
- Renames the `holochain_container` executable to simply `holochain` [#942](https://github.com/holochain/holochain-rust/pull/942)
- Renames the `cmd` crate (which implements the `hc` command line tool) to `cli` [#940](https://github.com/holochain/holochain-rust/pull/940)
- Encoded values in ribosome function's input/output are u64 (up from u32) [#915](https://github.com/holochain/holochain-rust/pull/915)
- Updated dependencies: [#924](https://github.com/holochain/holochain-rust/pull/924)
  * Rust nightly to `2019-01-24`
  * futures to `0.3.0-alpha.12`
- All chain headers are sent in the validation package, not just those for public entry types. [#926](https://github.com/holochain/holochain-rust/pull/926)
### Added
- Adds centralized documentation for environment variables in use by Holochain [#990](https://github.com/holochain/holochain-rust/pull/990)
- Adds command `hc keygen` which creates a new key pair, asks for a passphrase and writes an encrypted key bundle file to `~/.holochain/keys`. [#974](https://github.com/holochain/holochain-rust/pull/974)
- Adds an environment variable `NETWORKING_CONFIG_FILE` for specifing the location of the json file containing the network settings used by n3h. [#976](https://github.com/holochain/holochain-rust/pull/976)
- Adds an environment variable `HC_SIMPLE_LOGGER_MUTE` for use in testing which silences logging output so CI logs won't be too big. [#960](https://github.com/holochain/holochain-rust/pull/960)
- Adds Zome API function `hdk::sleep(std::time::Duration)` which works the same as `std::thread::sleep`.[#935](https://github.com/holochain/holochain-rust/pull/935)
- All structs/values to all HDK functions must implement `Into<JsonString>` and `TryFrom<JsonString>` (derive `DefaultJson` to do this automatically) [#854](https://github.com/holochain/holochain-rust/pull/854)
- HDK globals `AGENT_ADDRESS`, `AGENT_ID_STR`, `DNA_NAME` and `DNA_ADDRESS` are now set to real, correct values. [#796](https://github.com/holochain/holochain-rust/pull/796)
- `hc run` now looks for the --interface flag or `HC_INTERFACE` env var if you want to specify the `http` interface [#846]((https://github.com/holochain/holochain-rust/pull/846)
- NodeJS Conductor added to allow running conductors for testing purposes in JavaScript. [#1007](https://github.com/holochain/holochain-rust/pull/1007)
- Scenario API added to enable deterministic scenario tests for zome functions. See the [NodeJS Conductor README](nodejs_conductor/README.md) for details. [#942](https://github.com/holochain/holochain-rust/pull/942)
- `hdk::holochain_core_types::time::Iso8601` now supports validation and conversion to DateTime, and is sortable. [#917](https://github.com/holochain/holochain-rust/pull/917)
- `hdk::query_result` API supports return of ChainHeader and/or Entry data for the matched EntryType(s) [#868](https://github.com/holochain/holochain-rust/pull/868)
- Admin RPC functions added to container interface. Any (websocket) container interface that is configured with  `admin = true`  now can call a number of functions to remotely change any aspect of the container config. [#840](https://github.com/holochain/holochain-rust/pull/840)
- Adds a set of functions to the container RPC for managing static UI bundles and HTTP interfaces to these.  See rustdoc of `conductor_api::interface::ConductorApiBuilder` for a full description of these functions. [#919](https://github.com/holochain/holochain-rust/pull/919)
- Conductor can now serve static directories called ui_bundles over HTTP that can be configured in the container config toml file. This HTTP server also implements a virtual json file at "/_dna_connections.json" that returns the DNA interface (if any) the UI is configured to connect to. Hc-web-client will use this to automatically connect to the correct DNA interface on page load.  [#885](https://github.com/holochain/holochain-rust/pull/885)
- Adds Zome API function `hdk::remove_link(base,target,tag)` for removing links.  [#780](https://github.com/holochain/holochain-rust/pull/780)

## [0.0.3] - 2019-01-15
### Fixed
- build problems because of changes to upstream futures-preview crate [#864](https://github.com/holochain/holochain-rust/pull/864)
### Added
- Networking: beyond mock, using [n3h](https://github.com/holochain/n3h) [#831](https://github.com/holochain/holochain-rust/pull/831)
- Bridging now works and is configurable in the container (no capabilities yet) [#779](https://github.com/holochain/holochain-rust/pull/779) & [#776](https://github.com/holochain/holochain-rust/pull/776)
- Validation across network [#727](https://github.com/holochain/holochain-rust/pull/727)
- API/HDK: [#831](https://github.com/holochain/holochain-rust/pull/831)
    - CRUD for entries working
    - Node-to-node messaging [#746](https://github.com/holochain/holochain-rust/pull/746)
    - GetEntryOptions:
        - retrieve CRUD history & status
        - meta data: sources
    - GetLinksOptions
        - meta data: sources
    - GetLinks helpers: get_links_and_load
    - Query: return multiple entry types with glob matching [#781](https://github.com/holochain/holochain-rust/pull/781)
- Conductor: [#942](https://github.com/holochain/holochain-rust/pull/942)
    - configuration builder and config files
    - http interface [#823](https://github.com/holochain/holochain-rust/pull/823)
- hc command-line tool: [#831](https://github.com/holochain/holochain-rust/pull/831)
    - `run --persist` flag for keeping state across runs [#729](https://github.com/holochain/holochain-rust/pull/729/files)
    - Added env variables to activate real networking [#826](https://github.com/holochain/holochain-rust/pull/826)
- Groundwork for: capabilities & signals [#762](https://github.com/holochain/holochain-rust/pull/762) & [#732](https://github.com/holochain/holochain-rust/pull/732)
- Improved debug logging with log rules and colorization [#819](https://github.com/holochain/holochain-rust/pull/819)
- This change log! [#831](https://github.com/holochain/holochain-rust/pull/831)

### Changed
- API/HDK: [#831](https://github.com/holochain/holochain-rust/pull/831)
    - native return types (JsonStrings)
    - many places where we referred to "Hash" we now use the more correct term "Address"

## [0.0.2] - 2018-11-28
### Added
- mock networking [#831](https://github.com/holochain/holochain-rust/pull/831)
- `hc run` with support for [#831](https://github.com/holochain/holochain-rust/pull/831)
- multi-instance scenario testing [#831](https://github.com/holochain/holochain-rust/pull/831)
