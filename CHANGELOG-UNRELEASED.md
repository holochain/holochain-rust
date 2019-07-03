# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added
- Added `properties` to entry definitions (not to the entries themselved). These can be retrieved using the `entry_type_properties` HDK function [#1337](https://github.com/holochain/holochain-rust/pull/1337)
- *Breaking Change* Added type field to conductor network configuration.  You must add `type="n3h"` for current config files to work.  [#1540](https://github.com/holochain/holochain-rust/pull/1540)
- Added `Encryption` and `Decryption` methods in the HDK [#1534](https://github.com/holochain/holochain-rust/pull/1534)
- Adds `hc hash` CLI subcommand. Can be used to compute the hash of the DNA in the current dist directory or passed a path to a DNA with the --path flag [#1562](https://github.com/holochain/holochain-rust/pull/1562)
- Adds a --dna flag to the CLI so `hc run` can run DNAs outside the standard ./dist/ directory [1561](https://github.com/holochain/holochain-rust/pull/1561)

### Changed

### Deprecated

### Removed

### Fixed

### Security
