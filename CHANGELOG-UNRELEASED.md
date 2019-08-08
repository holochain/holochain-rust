# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

* New logging implementation added as a subcrate : a fast logger with a filtering capability using regex expressions, please so [logging](logging) for more details.
* Adds publishing of headers again after rollback. Header publishing is now its own action rather than part of the `Publish` action that plays nicely with the testing framework. It also adds header entries to the author list so they are gossiped properly. [#1640](https://github.com/holochain/holochain-rust/pull/1640).

### Changed

- Bump dependent crate versions (holochain_persistence 0.0.7, holochain_serialization 0.0.7, lib3h 0.0.10) in preparation futures 0.3.0-alpha17 which will allow us to shift to the upcoming Rust 1.38.0 beta [#1632](https://github.com/holochain/holochain-rust/pull/1632)

### Deprecated

### Removed

### Fixed

### Security
