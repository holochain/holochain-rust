# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added
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
