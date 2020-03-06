# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added
- Adds tokio tracing to sim2h_server. [Description](https://holo.hackmd.io/@c5lIpp4ET0OJJnDT3gzilA/SyRm2YoEU). Also check `sim2h_server --help` for usage instructions.
- Adds the notion of a manager to trycp_server so that we can dynamically manage pools of available nodes for test runs in final-exam  [PR#2123](https://github.com/holochain/holochain-rust/pull/2123)

### Changed
- new_relic is behind a feature flag `new-relic`.
### Deprecated

### Removed
- Older rust-tracing traces.

### Fixed
- Pagination option for get_links now retrieves entries before `from_time`, not after [PR#2144](https://github.com/holochain/holochain-rust/pull/2144)

### Security
