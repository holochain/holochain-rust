# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

### Changed

- Replace naive timeout implementation (for network queries / direct messages) that uses a thread per timeout with a scheduled job that polls the State and sends timeout actions when needed (reduces number of used threads and thus memory footprint) [#1916](https://github.com/holochain/holochain-rust/pull/1916).

### Deprecated

### Removed

### Fixed

### Security

