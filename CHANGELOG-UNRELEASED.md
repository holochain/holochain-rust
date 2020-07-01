# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

- restores changes from v0.0.48-alpha1 [2195](https://github.com/holochain/holochain-rust/pull/2195)
- Adds experimental debugging repl [2196](https://github.com/holochain/holochain-rust/pull/2196)
- Adds eavi to state dump [2196](https://github.com/holochain/holochain-rust/pull/2196)

### Changed

### Deprecated

### Removed

### Fixed

- Fixes state mismatch bugs [2196](https://github.com/holochain/holochain-rust/pull/2196)
- Fixes sending of already timed-out direct messages on sim2h reconnect bug [2196](https://github.com/holochain/holochain-rust/pull/2196)
- Fixes incorrect regeneration of remove link aspects from eavi.  [2196](https://github.com/holochain/holochain-rust/pull/2196)
- Validation no longer unnecessarily run twice when holding an add_link or a remove_link
- Increases timeout getting headers when building a validation package from the dht (1 second isn't enough) [#2199](https://github.com/holochain/holochain-rust/pull/2199)
- Pending items were being de-queued when processing, which meant that if another request arrived from sim2h (which could happen often under some circumstances), then the same item would be queued multiple times, which could snowball.  The solution was to move in-process items to another queue for checking. [#2199](https://github.com/holochain/holochain-rust/pull/2199)
- Fixed DHT queries for ChainHeader entries which were failing because there are no headers for headers stored in the DHT which the code was expecting. [#2199](https://github.com/holochain/holochain-rust/pull/2199)

### Security
