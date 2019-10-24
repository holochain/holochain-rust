# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

*  Adds the holochain_persistence_lmdb crate and makes this an option for the instance config. This is now the default store implementation. [#1758](https://github.com/holochain/holochain-rust/pull/1758)

### Changed

* Custom signals that are emitted from DNA/zome code ("user" signals) are now send to all admin interfaces to enable UI switching logic in Holoscape [#1799](https://github.com/holochain/holochain-rust/pull/1799)

### Deprecated

### Removed

### Fixed

### Security

