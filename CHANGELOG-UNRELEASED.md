# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added
* The sim2h switch-board server is now caching if a node is missing data and periodically checks back in. This makes it more resilient against unforseen problems like connection drops which otherwise could only be recovered through an explicit reconnection of the node. [#1834](https://github.com/holochain/holochain-rust/pull/1834) 

* Adds a clean option to the `admin/instance/remove` method on the `ConductorApiBuilder`. [#1775](https://github.com/holochain/holochain-rust/pull/1775).

### Changed

### Deprecated

### Removed

### Fixed

### Security

