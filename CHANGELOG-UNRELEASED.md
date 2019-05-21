# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

### Changed

- **Breaking Change** Renames link tags to link_type. Adds new link tag which can be any string. This is available in validation of links and links can be retrieved based on their tag+type, just tag, just type or retrieve all. `hdk::link_entries` and `hdk::get_links` now required an extra parameter.  [#1402](https://github.com/holochain/holochain-rust/pull/1402).

### Deprecated

### Removed

### Fixed

### Security
