# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

- Adds smarter ordering of pending validation. Builds a dependency graph and only will try to validate entries that do not have dependencies also awaiting validation as this will always fail. [#1924](https://github.com/holochain/holochain-rust/pull/1924)
- Add ability to validate entries with full chain validation when author is offline [#1932](https://github.com/holochain/holochain-rust/pull/1932)
- Adds support for [hApp-bundles](https://github.com/holochain/holoscape/tree/master/example-bundles) to `hc run`. This enables complex hApp setups with multiple DNAs and bridges to be easily run during development without having to write/maintain a conductor config file. [#1939](https://github.com/holochain/holochain-rust/pull/1939)

### Changed

### Deprecated

### Removed

### Fixed

### Security

