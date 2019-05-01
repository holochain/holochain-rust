# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

- Adds hdk::commit_capability_grant() for zome functions to be able to create [capability grant](doc/architecture/decisions/0017-capabilities.md)  [#1285](https://github.com/holochain/holochain-rust/pull/1285)
- Adds hdk::commit_entry_result() which features: optional argument to include additional provenances. [#1320](https://github.com/holochain/holochain-rust/pull/1320)

### Changed
- Changes `LinkAdd` and `RemoveEntry` so that they return a hash instead of a null [#1343](https://github.com/holochain/holochain-rust/pull/1343)

### Deprecated

### Removed

### Fixed

### Security
