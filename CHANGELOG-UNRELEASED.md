# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

- Added command to sim2h wire protocol for getting live debug info [#2128](https://github.com/holochain/holochain-rust/pull/2128)
- Added an environment variable (HC_IGNORE_SIM2H_URL_PROPERTY) which overrides DNA sim2h_url value for running conductors in test modes

### Changed

- Changed Pagination to have different types [#2110](https://github.com/holochain/holochain-rust/pull/2110)
- Link matches are not based on regex anymore [#2133](https://github.com/holochain/holochain-rust/pull/2133)

### Deprecated

### Removed

### Fixed
- Make Holochain (i.e. Sim2hWorker) work offline again (that is without being connected to Sim2h) [#2119](https://github.com/holochain/holochain-rust/pull/2119)

- Fixing wire message TK-01102
- Fixed `panic!("entry/aspect mismatch - corrupted data?")` [#2135](https://github.com/holochain/holochain-rust/pull/2135)

### Security
