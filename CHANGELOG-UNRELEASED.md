# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

### Changed

- Exchanged [vanilla thread-pool](https://docs.rs/threadpool/1.7.1/threadpool/) with the futures executor thread-pool [from the futures crate](https://docs.rs/futures/0.3.1/futures/executor/index.html). This enables M:N Future:Thread execution which is much less wasteful than having a thread per future. Number of threads in the pool is kept at the default (of that crate) of number of CPUs. [#1915](https://github.com/holochain/holochain-rust/pull/1915) 
- Replace naive timeout implementation (for network queries / direct messages) that uses a thread per timeout with a scheduled job that polls the State and sends timeout actions when needed (reduces number of used threads and thus memory footprint) [#1916](https://github.com/holochain/holochain-rust/pull/1916).
- Use the [im crate](https://docs.rs/im/14.0.0/im/) for `HashMap`s and `HashSet`s used in the redux State. This makes cloning the state much cheaper and improves over-all performance. [#1923](https://github.com/holochain/holochain-rust/pull/1923)

### Deprecated

### Removed

### Fixed

### Security

