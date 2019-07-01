# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added
- *Breaking Change* Added type field to conductor network configuration.  You must add `type="n3h"` for current config files to work.  [#1540](https://github.com/holochain/holochain-rust/pull/1540)
- Added `Encryption` and `Decryption` methods in the HDK [#1534](https://github.com/holochain/holochain-rust/pull/1534)
- Adds a --dna flag to the CLI so `hc run` can run DNAs outside the standard ./dist/ directory [1561](https://github.com/holochain/holochain-rust/pull/1561)

<<<<<<< HEAD
- **Breaking change** - renames `emit_trace_signals` to `signals.trace` in conductor config [#1431](https://github.com/holochain/holochain-rust/pull/1431)
- "Consistency" signals added, which aid determinism in end-to-end tests, configurable through `signals.consistency` conductor config [#1431](https://github.com/holochain/holochain-rust/pull/1431)
- Uses regex matching for `get_links` tags and type. Probably not a breaking change but be careful of subset matching (e.g. `some` will match against `some-tag` but `^some$` will not.) [#1453](https://github.com/holochain/holochain-rust/pull/1453)
- **Breaking Change** genesis function now renamed to init [#1417](https://github.com/holochain/holochain-rust/pull/1417)
=======
### Changed
>>>>>>> d230e90bb4be997b0cc4a1a015e7586610480f09

### Deprecated

### Removed

### Fixed

### Security
