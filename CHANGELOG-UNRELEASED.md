# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

- **Breaking change** - renames `emit_trace_signals` to `signals.trace` in conductor config [#1431](https://github.com/holochain/holochain-rust/pull/1431)
- "Consistency" signals added, which aid determinism in end-to-end tests, configurable through `signals.consistency` conductor config [#1431](https://github.com/holochain/holochain-rust/pull/1431)
- Uses regex matching for `get_links` tags and type. Probably not a breaking change but be careful of subset matching (e.g. `some` will match against `some-tag` but `^some$` will not.) [#1453](https://github.com/holochain/holochain-rust/pull/1453)
- `Tombstone` functionality added on eaviquery, this makes sure that the delete links is not determined by order but determined by a `tombstone set` which takes precedence over everything. [#1363](https://github.com/holochain/holochain-rust/pull/1363)

### Deprecated

### Removed

- **Breaking change** - migrates nodejs_conductor and nodejs_waiter to holochain-nodejs repo [#1510](https://github.com/holochain/holochain-rust/pull/1510)

### Fixed

### Security
