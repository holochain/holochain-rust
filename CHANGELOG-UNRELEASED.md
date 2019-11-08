# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added
* Adds try-o-rama remote server provisioning via trycp [#1780](https://github.com/holochain/holochain-rust/pull/1780)
  This also adds nix-shell commands:
  - `hc-trycp-server-install` which installs the trycp-server
  - `hc-trycp-server` which runs the trycp-server
* Adds instrumentation to measure and publish. performance. Introduces `hc-metrics` command to parse logs and generate statistics. [#1810](https://github.com/holochain/holochain-rust/pull/1810)

### Changed

### Deprecated

### Removed

### Fixed

### Security
