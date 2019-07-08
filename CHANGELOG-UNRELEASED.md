# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added
-- Added a `get_links_count` method which allows the user to get number of links by base and tag [#1568](https://github.com/holochain/holochain-rust/pull/1568)

### Changed

- *Breaking Change* Validation callback now shows consistent behavior when called on the authoring node during entry commit time, and when called by validating nodes being requested to hold the entry.  In both cases the a FullChain validation package now does NOT include the about-to-be-added entry.  Some validation functions were relying on the behavior of having the entry be at the top of the chain in the Hold case, and using the EntryLifecycle flag value to distinguish the two cases.   Please note that in the future this flag may be going away! [#1563](https://github.com/holochain/holochain-rust/pull/1563)

### Deprecated

- The EntryLifecycle flags in validation may be going away.  If you have a use-case that requires this, please tell us.

### Removed

### Fixed

### Security
