# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added
- Error log output added for errors occurring during `hdk::call`, including bridge call errors [#1448](https://github.com/holochain/holochain-rust/pull/1448).

### Changed
- Added a Vagrant file to support nix-shell compatible VMs on windows etc. [#1433](https://github.com/holochain/holochain-rust/pull/1433)
- Adds TryInto implementation from JsonString to generic result types. This makes bridge calls much easier to implement safely [#1464](https://github.com/holochain/holochain-rust/pull/1464)

- **Breaking change** - renames `emit_trace_signals` to `signals.trace` in conductor config [#1431](https://github.com/holochain/holochain-rust/pull/1431)
- "Consistency" signals added, which aid determinism in end-to-end tests, configurable through `signals.consistency` conductor config [#1431](https://github.com/holochain/holochain-rust/pull/1431)
- Uses regex matching for `get_links` tags and type. Probably not a breaking change but be careful of subset matching (e.g. `some` will match against `some-tag` but `^some$` will not.) [#1453](https://github.com/holochain/holochain-rust/pull/1453)

### Deprecated

### Removed

### Fixed

### Security
