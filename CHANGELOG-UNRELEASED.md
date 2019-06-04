# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added
- Error log output added for errors occurring during `hdk::call`, including bridge call errors [#1448](https://github.com/holochain/holochain-rust/pull/1448).
- New `uuid` parameter for `admin/dna/install_from_file`, to set the UUID of the installed DNA, changing its hash

### Changed
- Added a Vagrant file to support nix-shell compatible VMs on windows etc. [#1433](https://github.com/holochain/holochain-rust/pull/1433)
- Adds TryInto implementation from JsonString to generic result types. This makes bridge calls much easier to implement safely [#1464](https://github.com/holochain/holochain-rust/pull/1464)

### Changed

### Deprecated

### Removed

### Fixed

- Adding bridges dynamically via an admin interface works now without rebooting the conductor. [#1476](https://github.com/holochain/holochain-rust/pull/1476) 

### Security
