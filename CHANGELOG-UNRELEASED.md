# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

- Added get_meta to admin interface for access to DHT info  [#2207](https://github.com/holochain/holochain-rust/pull/2208)
- Sim2h messages now ssl encoded for security and simplify integrity so conductor only needs to sign JOIN message  [#2203](https://github.com/holochain/holochain-rust/pull/2203)
- Adds conductor signing-service error.  [#2203](https://github.com/holochain/holochain-rust/pull/2203)

### Changed

### Deprecated

### Removed

### Fixed

- Fixed memory leak in various timeout conditions of direct messages  [#2208](https://github.com/holochain/holochain-rust/pull/2208)
- Fixed holding-list/CAS mismatch potential in various error conditions when sim2h requests aspect holding [#2208](https://github.com/holochain/holochain-rust/pull/2208)
- Fixed bug in holding remove link before add-link validation complete [#2208](https://github.com/holochain/holochain-rust/pull/2208)

### Security
