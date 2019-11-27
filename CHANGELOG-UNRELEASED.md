# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

### Changed

- Use the [im crate](https://docs.rs/im/14.0.0/im/) for `HashMap`s and `HashSet`s used in the redux State. This makes cloning the state much cheaper and improves over-all performance. [#1923](https://github.com/holochain/holochain-rust/pull/1923)

### Deprecated

### Removed

### Fixed

### Security

