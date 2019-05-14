# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

- Adds new RPC method to conductor `test/agent/add` which adds an agent but does not save the config or generate a keystore. This is added to enable tests that run against the Rust conductor [PR#1359](https://github.com/holochain/holochain-rust/pull/1359)
- Adds `from` argument to the `receive` callback. [#1382](https://github.com/holochain/holochain-rust/pull/1382)
- Adds a new hdk::keystore_get_public_key function which returns the public key of a key secret from the keystore.
- Adds hdk::commit_capability_grant() for zome functions to be able to create [capability grant](doc/architecture/decisions/0017-capabilities.md)  [#1285](https://github.com/holochain/holochain-rust/pull/1285)
- Adds hdk::commit_entry_result() which features: optional argument to include additional provenances. [#1320](https://github.com/holochain/holochain-rust/pull/1320)
- conductor now persists its config in the config root (e.g. `home/peter/.config/holochain/conductor`) rather than `~/.holochain` [#1386](https://github.com/holochain/holochain-rust/pull/1386)

### Changed

### Deprecated

### Removed

### Fixed

### Security


