# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

- Adds a retry if a net worker cannot be spawned on startup [#1870](https://github.com/holochain/holochain-rust/pull/1870)
- Add hdk::version_hash, returning MD5 hash of HDK build environment [#1869](https://github.com/holochain/holochain-rust/pull/1869)

### Changed

### Deprecated

### Removed

### Fixed

- Fix lots of deadlocks by managing threads and encapsulating locks [#1852](https://github.com/holochain/holochain-rust/pull/1852)
- Have sim2h let go of nodes if the connection got lost because of an error [#1877](https://github.com/holochain/holochain-rust/pull/1877)
### Security

