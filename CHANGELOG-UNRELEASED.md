# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added
- Added `Crud Status` information to link data in get_links as well as query through `LinkStatusRequest` [#1337](https://github.com/holochain/holochain-rust/pull/1337)
- Added a MVP implementation of [Signals](https://github.com/holochain/holochain-rust/blob/develop/doc/architecture/decisions/0013-signals-listeners-model-and-api.md) that introduces `hdk::emit_signal(name, payload)` [#1516](https://github.com/holochain/holochain-rust/pull/1516)

### Changed
- The barebones tests produced by `hc init` now use the Diorama testing framework rather than holochain-nodejs [#1532](https://github.com/holochain/holochain-rust/pull/1532)

### Deprecated

### Removed

### Fixed

### Security

