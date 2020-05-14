# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

- added an optimization of not querying the DHT for get links requests that are placed on a node's own agent id because we must be responsible for holding that part of the DHT. [#2189](https://github.com/holochain/holochain-rust/pull/2189)
- added an optimization for sim2h to not re-fetch entries we've fetched within the last second. [#2185](https://github.com/holochain/holochain-rust/pull/2185)

### Changed

### Deprecated

### Removed

### Fixed

- two conductor fixes during error cases (like base hasn't arrived yet) in hold_aspects request: [#2184](https://github.com/holochain/holochain-rust/pull/2184)
    - stop incorrectly recording aspects as held
    - stop locking up the future
- because sim2h doesn't get error messages from a hold_aspect request it must also not record aspects sent in those messages as held, and thus the conductor must explicitly send back a list of aspects held after a hold_aspect request [#2184](https://github.com/holochain/holochain-rust/pull/2184)
- update futures crate because of dependency issues.  [#2188](https://github.com/holochain/holochain-rust/pull/2188)

### Security
