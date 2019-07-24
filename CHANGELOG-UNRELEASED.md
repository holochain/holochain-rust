# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

### Changed
- **Breaking Change** genesis function now renamed to init [#1508](https://github.com/holochain/holochain-rust/pull/1508)
- **BREAKING:** Zomes must now include a `validate_agent` callback. If this rejects in any zome the DNA will not start. This can be used to enforce membrane requirements. [#1497](https://github.com/holochain/holochain-rust/pull/1497)
- Added a `get_links_count` method which allows the user to get number of links by base and tag [#1568](https://github.com/holochain/holochain-rust/pull/1568)### Changed
- The Conductor will shut down gracefully when receiving SIGINT (i.e. Ctrl+C) or SIGKILL, also causing a graceful shutdown of an attached n3h instance, if running [#1599](https://github.com/holochain/holochain-rust/pull/1599)

### Deprecated

### Removed

### Fixed
- Fixed problem with `hc run` that was introduced by [Conductor config sanitizing](https://github.com/holochain/holochain-rust/pull/1335) a week ago: The conductor config now needs to include the correct hash of each configured DNA file. [#1603](https://github.com/holochain/holochain-rust/pull/1603) adds the proper hash to the internally created conductor config that `hc run` runs.

### Security

