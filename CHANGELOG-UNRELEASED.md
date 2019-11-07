# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added
* The sim2h switch-board server is now caching if a node is missing data and periodically checks back in. This makes it more resilient against unforseen problems like connection drops which otherwise could only be recovered through an explicit reconnection of the node. [#1834](https://github.com/holochain/holochain-rust/pull/1834) 

### Changed
* DNA is now checked for invalid zome artifacts. Validation callbacks that fail unexpectedly will now panic rather than fail validation. `hc package` `--strip-meta` flag is now `--include-meta`. [#1838](https://github.com/holochain/holochain-rust/pull/1838) 

### Deprecated

### Removed

### Fixed

* Loading of instances from storage was broken and ended up in partially loaded states. This got fixed with [#1836](https://github.com/holochain/holochain-rust/pull/1836).

### Security

