# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

* New logging implementation added as a subcrate : a fast logger with a filtering capability using regex expressions, please so [logging](logging) for more details. [#1537](https://github.com/holochain/holochain-rust/pull/1537) and [#1639](https://github.com/holochain/holochain-rust/pull/1639)
* Ability to provide passphrase to lock/unlock keystores via IPC unix domain socket added. [#1646](https://github.com/holochain/holochain-rust/pull/1646) 

### Changed

- Bump dependent crate versions (holochain_persistence 0.0.7, holochain_serialization 0.0.7, lib3h 0.0.10) in preparation futures 0.3.0-alpha17 which will allow us to shift to the upcoming Rust 1.38.0 beta [#1632](https://github.com/holochain/holochain-rust/pull/1632)

### Deprecated

### Removed

### Fixed

### Security
