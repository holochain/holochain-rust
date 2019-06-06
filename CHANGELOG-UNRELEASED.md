# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added
- Error log output added for errors occurring during `hdk::call`, including bridge call errors [#1448](https://github.com/holochain/holochain-rust/pull/1448).
- **BREAKING:** Conductor configuration checks for bridges added [#1461](https://github.com/holochain/holochain-rust/pull/1461). Conductor will bail with an error message if the configuration of bridges between instances does not match the bridge requirements defined in the caller instance's DNA (required bridge missing, DNA hash mismatch, trait mismatch) or if a bridge with the handle specified in the config can not be found in the caller's DNA. 
- Added `Crud Status` information to link data in get_links as well as query through `LinkStatusRequest` [#1337](https://github.com/holochain/holochain-rust/pull/1337)
### Changed
- Added a Vagrant file to support nix-shell compatible VMs on windows etc. [#1433](https://github.com/holochain/holochain-rust/pull/1433)
- Adds TryInto implementation from JsonString to generic result types. This makes bridge calls much easier to implement safely [#1464](https://github.com/holochain/holochain-rust/pull/1464)

### Changed

### Deprecated

### Removed

### Fixed

- Adding bridges dynamically via an admin interface works now without rebooting the conductor. [#1476](https://github.com/holochain/holochain-rust/pull/1476)
- `hdk::query` results are filtered now to not contain DNA entries since they can easily be several MBs of size which breaks our current limitation of 640k of WASM memory. [#1490](https://github.com/holochain/holochain-rust/pull/1490)   

### Security
