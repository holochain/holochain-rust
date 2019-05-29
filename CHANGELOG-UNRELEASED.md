# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

- Error log output added for errors occurring during `hdk::call`, including bridge call errors [#1448](https://github.com/holochain/holochain-rust/pull/1448).

### Changed

- **Breaking change** - renames `emit_trace_signals` to `signals.trace` in conductor config [#1431](https://github.com/holochain/holochain-rust/pull/1431)
- "Consistency" signals added, which aid determinism in end-to-end tests, configurable through `signals.consistency` conductor config [#1431](https://github.com/holochain/holochain-rust/pull/1431)

### Deprecated

### Removed

### Fixed

### Security


