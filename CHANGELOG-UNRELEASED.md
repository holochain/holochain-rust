# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added
- Adds header publishing. Headers are published as entries to the DHT for all commits to local chain. This will eventually make local chains recoverable. [#1615](https://github.com/holochain/holochain-rust/pull/1615)
- 
### Changed
- The static file server has been replaced and now uses the Nickel crate intead of Hyper. It now correctly sets content type headers and can be configured to bind to a different address in the conductor config toml [#1595](https://github.com/holochain/holochain-rust/pull/1595)
- Optimized get_links so that fewer network calls are made overrall [#1607](https://github.com/holochain/holochain-rust/pull/1607)

- DEPRECATION WARNING, conductor static UI server is to be removed in an upcoming release. Devs will receive a warning when starting a conductor with a UI server configured [PR#1602](https://github.com/holochain/holochain-rust/pull/1602)

### Deprecated

### Removed

### Fixed

### Security

