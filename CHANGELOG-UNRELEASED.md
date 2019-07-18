# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

### Changed
- **Breaking Change** genesis function now renamed to init [#1417](https://github.com/holochain/holochain-rust/pull/1417)
- **BREAKING:** Zomes must now include a `validate_agent` callback. If this rejects in any zome the DNA will not start. This can be used to enforce membrane requirements. [#1497](https://github.com/holochain/holochain-rust/pull/1497)
- Added a `get_links_count` method which allows the user to get number of links by base and tag [#1568](https://github.com/holochain/holochain-rust/pull/1568)### Changed

### Deprecated

### Removed

### Fixed

### Security

