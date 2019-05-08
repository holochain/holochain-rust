# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

- Adds a new hdk::keystore_get_public_key function which returns the public key of a key secret from the keystore.
- Adds hdk::commit_capability_grant() for zome functions to be able to create [capability grant](doc/architecture/decisions/0017-capabilities.md)  [#1285](https://github.com/holochain/holochain-rust/pull/1285)
- Adds hdk::commit_entry_result() which features: optional argument to include additional provenances. [#1320](https://github.com/holochain/holochain-rust/pull/1320)
- Adds new RPC method to conductor `test/agent/add` which adds an agent but does not save the config or generate a keystore. This is added to enable tests that run against the Rust conductor [PR#1359](https://github.com/holochain/holochain-rust/pull/1359)

### Changed

- Updated linked [n3h](https://github.com/holochain/n3h) version to v0.0.12-alpha [#1369](https://github.com/holochain/holochain-rust/pull/1369)
- pin mozilla overlay to latest commit in nix [#1375](https://github.com/holochain/holochain-rust/pull/1375)

### Deprecated

### Removed

### Fixed

### Security
