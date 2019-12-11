# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

- Adds support for [hApp-bundles](https://github.com/holochain/holoscape/tree/master/example-bundles) to `hc run`. This enables complex hApp setups with multiple DNAs and bridges to be easily run during development without having to write/maintain a conductor config file. [#1939](https://github.com/holochain/holochain-rust/pull/1939)
- Adds ability to validate entries with full chain validation when author is offline [#1932](https://github.com/holochain/holochain-rust/pull/1932)
- Adds a conductor level stats-signal that sends an overview of instance data (number of held entries etc.) over admin interfaces. [#1954](https://github.com/holochain/holochain-rust/pull/1954)
- Adds parameters to conductor RPC function `debug/state_dump` to select portions of the state to be send instead of always receiving the full dump (which can get big if the instance holds many entries). [#1954](https://github.com/holochain/holochain-rust/pull/1954) 
### Changed

### Deprecated

### Removed

### Fixed

### Security

