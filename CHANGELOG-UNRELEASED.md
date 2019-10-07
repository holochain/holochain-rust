# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

*  Adds the `--properties`/`-p` flag to `hc package` which takes a stringifed JSON object to be inserted in the .dna.json under the properties field. This will alter the DNA hash and can therefore be used for fork DNAs from their source code. [#1720](https://github.com/holochain/holochain-rust/pull/1720)
* Adds publishing of headers again after rollback. Header publishing is now its own action rather than part of the `Publish` action that plays nicely with the testing framework. It also adds header entries to the author list so they are gossiped properly. [#1640](https://github.com/holochain/holochain-rust/pull/1640).
* Adds some deadlock diagnostic tools to detect when any mutex has been locked for a long time, and prints the backtrace of the moment it was locked [#1743](https://github.com/holochain/holochain-rust/pull/1743)

### Changed

* Updates to work with latest version of lib3h  [#1737](https://github.com/holochain/holochain-rust/pull/1737)

### Deprecated

### Removed

### Fixed

### Security
