# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

### Changed

- Adds `--uuid` flag to `hc hash`, allowing a UUID to be specified which will alter the hash [PR#2161](https://github.com/holochain/holochain-rust/pull/2161)
- Adds `--files` flag to `hc sim2h-client`, which when set prints JSON blobs to multiple files named by space hash (the previous default behavior), and when unset prints a single JSON blob to stdout for easy parsing by script [PR#2161](https://github.com/holochain/holochain-rust/pull/2161)

### Deprecated

### Removed

### Fixed
- Bug where EntryAspects were being associated with the wrong Entries

### Security

