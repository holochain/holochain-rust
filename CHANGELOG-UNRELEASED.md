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

### Security
