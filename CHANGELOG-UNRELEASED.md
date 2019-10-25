# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added
* Adds try-o-rama remote server provisioning via trycp [#1780](https://github.com/holochain/holochain-rust/pull/1780)
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

### Security
