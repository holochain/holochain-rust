# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added
- Error log output added for errors occurring during `hdk::call`, including bridge call errors [#1448](https://github.com/holochain/holochain-rust/pull/1448).
- New `uuid` parameter for `admin/dna/install_from_file`, to set the UUID of the installed DNA, changing its hash
- **BREAKING:** Conductor configuration checks for bridges added [#1461](https://github.com/holochain/holochain-rust/pull/1461). Conductor will bail with an error message if the configuration of bridges between instances does not match the bridge requirements defined in the caller instance's DNA (required bridge missing, DNA hash mismatch, trait mismatch) or if a bridge with the handle specified in the config can not be found in the caller's DNA. 
- **BREAKING:** Zomes must now include a `validate_agent` callback. If this rejects in any zome the DNA will not start. This can be used to enforce membrane requirements. [#1497](https://github.com/holochain/holochain-rust/pull/1497)

### Changed
- Added a Vagrant file to support nix-shell compatible VMs on windows etc. [#1433](https://github.com/holochain/holochain-rust/pull/1433)
- Adds TryInto implementation from JsonString to generic result types. This makes bridge calls much easier to implement safely [#1464](https://github.com/holochain/holochain-rust/pull/1464)
- Changes the responses when using `hdk::call` to call across a bridge to make it consistent with calling between zomes  [#1487](https://github.com/holochain/holochain-rust/pull/1487)


### Changed

### Deprecated

### Removed

### Fixed

### Security


