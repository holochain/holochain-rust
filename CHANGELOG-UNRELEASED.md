# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

### Changed

- Exchanged [vanilla thread-pool](https://docs.rs/threadpool/1.7.1/threadpool/) with the futures executor thread-pool [from the futures crate](https://docs.rs/futures/0.3.1/futures/executor/index.html). This enables M:N Future:Thread execution which is much less wasteful than having a thread per future. Number of threads in the pool is kept at the default (of that crate) of number of CPUs. [#1915](https://github.com/holochain/holochain-rust/pull/1915) 

### Deprecated

### Removed

### Fixed

### Security

