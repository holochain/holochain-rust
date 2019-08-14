# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added
* Ability to provide passphrase to lock/unlock keystores via IPC unix domain socket added. [#1646](https://github.com/holochain/holochain-rust/pull/1646) 

* New logging implementation added as a subcrate : a fast logger with a filtering capability using regex expressions, please so [logging](logging) for more details.
* Adds publishing of headers again after rollback. Header publishing is now its own action rather than part of the `Publish` action that plays nicely with the testing framework. It also adds header entries to the author list so they are gossiped properly. [#1640](https://github.com/holochain/holochain-rust/pull/1640).
* Documentation for our links ecosystem [#1628](https://github.com/holochain/holochain-rust/pull/1628)

### Changed

### Deprecated

### Removed

### Fixed

### Security

