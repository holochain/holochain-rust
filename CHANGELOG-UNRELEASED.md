# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

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

