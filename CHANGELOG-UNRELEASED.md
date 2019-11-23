# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

- Adds a retry if a net worker cannot be spawned on startup [#1870](https://github.com/holochain/holochain-rust/pull/1870)
- Add hdk::version_hash, returning MD5 hash of HDK build environment [#1869](https://github.com/holochain/holochain-rust/pull/1869)
- Ability to set storage backend for new instances over RPC [#1900](https://github.com/holochain/holochain-rust/pull/1900)

### Changed

- Several improvements to gossip related code, both in sim2h server and core [#1884](https://github.com/holochain/holochain-rust/pull/1884/files):
  * Sim2h server will not just randomly pick a node to fill missing aspects, but it caches the information which aspects are missing for which node and will not ask a node about an aspect it doesn't have (gets rid of the `EntryNotFoundLocally` error).
  * In core's list responses: merge authoring list into the gossip list so sim2h has gossip sources that are the authors of entry aspects.
  * Clear sim2h server's caches about nodes when they disconnect. Also forget the whole space when the last node disconnectse.
 

### Deprecated

### Removed

### Fixed

- Fix lots of deadlocks by managing threads and encapsulating locks [#1852](https://github.com/holochain/holochain-rust/pull/1852)
- Have sim2h let go of nodes if the connection got lost because of an error [#1877](https://github.com/holochain/holochain-rust/pull/1877)
### Security

