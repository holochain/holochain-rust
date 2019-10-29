# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

*  Allows the HC CLI to generate zomes from template repos. This will by default use the default holochain template repos (holochain/rust-zome-template and holochain/rust-proc-zome-template) but can also point to custom templates in remote repos or locally (e.g. `hc generate zomes/my_zome https://github.com/org/some-custom-zome-template`). [#1565](https://github.com/holochain/holochain-rust/pull/1565)
* Adds option `--property` to `hc hash` that sets DNA properties for hash calculation. [#1807](https://github.com/holochain/holochain-rust/pull/1807)
### Changed

### Deprecated

### Removed

### Fixed

### Security

