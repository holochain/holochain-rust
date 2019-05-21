# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

- Option to show NPM output when pulling deps during `hc test` [PR#1401](https://github.com/holochain/holochain-rust/pull/1401)
- Adds scaffolding/skeleton for a future WASM conductor [#894](https://github.com/holochain/holochain-rust/pull/894)

### Changed

- Renames link tags to link_type. Adds new link tag which can be any string. This is available in validation of links and links can be retrieved based on their tag+type, just tag, just type or retrieve all.  [#1402](https://github.com/holochain/holochain-rust/pull/1402).
- Conductor now persists its config in the config root (e.g. `home/peter/.config/holochain/conductor` rather than `~/.holochain`) [#1386](https://github.com/holochain/holochain-rust/pull/1386)
- Default N3H mode as set when spawned by the conductor got set to "REAL". [#1282](https://github.com/holochain/holochain-rust/pull/1282)

### Deprecated

### Removed

### Fixed

### Security
