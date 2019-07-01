# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added
- **BREAKING:** Zomes must now include a `validate_agent` callback. If this rejects in any zome the DNA will not start. This can be used to enforce membrane requirements. [#1497](https://github.com/holochain/holochain-rust/pull/1497)
- *Breaking Change* Added type field to conductor network configuration.  You must add `type="n3h"` for current config files to work.  [#1540](https://github.com/holochain/holochain-rust/pull/1540)
- Added `Encryption` and `Decryption` methods in the HDK [#1534](https://github.com/holochain/holochain-rust/pull/1534)
- Adds a --dna flag to the CLI so `hc run` can run DNAs outside the standard ./dist/ directory [1561](https://github.com/holochain/holochain-rust/pull/1561)

### Changed

### Deprecated

### Removed

### Fixed

### Security
