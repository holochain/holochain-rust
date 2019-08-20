# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

* Conductor API debug functions added: 
    * `debug/running_instances`: returns array of running instance IDs
    * `debug/state_dump`: returns a state dump for a given instance
    * `debug/fetch_cas`: returns the content for a given entry address and instance ID
  
  Also added the source to the state dump.
  [#1661](https://github.com/holochain/holochain-rust/pull/1661)

### Changed

### Deprecated

### Removed

### Fixed

### Security

