# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

*  Allows the HC CLI to generate zomes from template repos. This will by default use the default holochain template repos (holochain/rust-zome-template and holochain/rust-proc-zome-template) but can also point to custom templates in remote repos or locally (e.g. `hc generate zomes/my_zome https://github.com/org/some-custom-zome-template`). [#1565](https://github.com/holochain/holochain-rust/pull/1565)
* Adds option `--property` to `hc hash` that sets DNA properties for hash calculation. [#1807](https://github.com/holochain/holochain-rust/pull/1807)
* Adds a prelude module to the HDK. Adding the statement `use hdk::prelude::*` should be enough for 90% of zome development [#1816](https://github.com/holochain/holochain-rust/pull/1816)
* Adds a special DNA property sim2h_url that, if set, overrides the conductor wide setting for the network configuration variable sim2h_url. [PR#1828](https://github.com/holochain/holochain-rust/pull/1828)
* Adds ability to download DNA from URL rather than pointing to local filesystem

### Changed

### Deprecated

* `file` key of DNA configuration is deprecated. It is now called `location`, and can refer to either a local file path or an HTTP(S) URL.

### Removed

### Fixed

* Fixes handling if DNA properties during `hc package`. DNA properties mentioned in the DNA's JSON manifest are now included in the package. [PR#1828](https://github.com/holochain/holochain-rust/pull/1828)  

### Security

