# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- Node JS version relaxed to 8.x in nix shell
- develop docker tag now uses nix
- legacy docker files removed
- nixos friendly shebang added to bash scripts
### Removed
### Added
- Adds a panic handler to HDK-Rust and that reroutes infos about panics happening inside the WASM Ribosome to the instances logger [PR#1029](https://github.com/holochain/holochain-rust/pull/1029).
- mac os x install script installs cmake and qt
-Added CrudStatus working over network [#1048] https://github.com/holochain/holochain-rust/pull/1048
### Fixed

## [0.0.4-alpha] - 2019-02-15

### Fixed
- Futures handling and zome function execution refactored which enables using complex API functions like `commit_entry` in callbacks such as `receive`.  This also fixes long standing flaky tests and blocking behaviors we have been experiencing. [#991](https://github.com/holochain/holochain-rust/pull/991)
### Changed
- Capabilities now separated from function declarations and renamed to `traits` in `define_zome!` and calling zome functions no longer uses capability name parameter [#997](https://github.com/holochain/holochain-rust/pull/997) & [#791](https://github.com/holochain/holochain-rust/pull/791)
- `hash` properties for `UiBundleConfiguration` and `DnaConfiguration` in Conductor config files is now optional
- `ChainHeader::sources()` is now `ChainHeader::provenances()` which stores both source address, and signature  [#932](https://github.com/holochain/holochain-rust/pull/932)
- `hdk::get_entry_results` supports return of ChainHeaders for all agents who have committed the same entry [#932](https://github.com/holochain/holochain-rust/pull/932)
- Renames the term Container and all references to it to Conductor [#942](https://github.com/holochain/holochain-rust/pull/942)
- Renames the `holochain_container` executable to simply `holochain`
- Renames the `cmd` crate (which implements the `hc` command line tool) to `cli` [#940](https://github.com/holochain/holochain-rust/pull/940)
- Encoded values in ribosome function's input/output are u64 (up from u32)
- Updated dependencies:
  * Rust nightly to `2019-01-24`
  * futures to `0.3.0-alpha.12`
- All chain headers are sent in the validation package, not just those for public entry types. [#926](https://github.com/holochain/holochain-rust/pull/926)
### Added
- Adds centralized documentation for environment variables in use by Holochain [#990](https://github.com/holochain/holochain-rust/pull/990)
- Adds command `hc keygen` which creates a new key pair, asks for a passphrase and writes an encrypted key bundle file to `~/.holochain/keys`. [#974](https://github.com/holochain/holochain-rust/pull/974)
- Adds an environment variable NETWORKING_CONFIG_FILE for specifing the location of the json file containing the network settings used by n3h.
- Adds an environment variable HC_SIMPLE_LOGGER_MUTE for use in testing which silences logging output so CI logs won't be too big.
- Adds Zome API function `hdk::sleep(std::time::Duration)` which works the same as `std::thread::sleep`.[#935](https://github.com/holochain/holochain-rust/pull/935)
- All structs/values to all HDK functions must implement `Into<JsonString>` and `TryFrom<JsonString>` (derive `DefaultJson` to do this automatically)
- HDK globals `AGENT_ADDRESS`, `AGENT_ID_STR`, `DNA_NAME` and `DNA_ADDRESS` are now set to real, correct values.
- `hc run` now looks for the --interface flag or `HC_INTERFACE` env var if you want to specify the `http` interface [#846]((https://github.com/holochain/holochain-rust/pull/846)
- NodeJS Conductor added to allow running conductors for testing purposes in JavaScript.
- Scenario API added to enable deterministic scenario tests for zome functions. See the [NodeJS Conductor README](nodejs_conductor/README.md) for details.
- `hdk::holochain_core_types::time::Iso8601` now supports validation and conversion to DateTime, and is sortable. [#917](https://github.com/holochain/holochain-rust/pull/917)
- `hdk::query_result` API supports return of ChainHeader and/or Entry data for the matched EntryType(s) [#868](https://github.com/holochain/holochain-rust/pull/868)
- Admin RPC functions added to container interface. Any (websocket) container interface that is configured with  `admin = true`  now can call a number of functions to remotely change any aspect of the container config. [#840](https://github.com/holochain/holochain-rust/pull/840)
- Adds a set of functions to the container RPC for managing static UI bundles and HTTP interfaces to these.  See rustdoc of `conductor_api::interface::ConductorApiBuilder` for a full description of these functions. [#919](https://github.com/holochain/holochain-rust/pull/919)
- Conductor can now serve static directories called ui_bundles over HTTP that can be configured in the container config toml file. This HTTP server also implements a virtual json file at "/_dna_connections.json" that returns the DNA interface (if any) the UI is configured to connect to. Hc-web-client will use this to automatically connect to the correct DNA interface on page load.  [#885](https://github.com/holochain/holochain-rust/pull/885)
- Adds Zome API function `hdk::remove_link(base,target,tag)` for removing links.  [#780](https://github.com/holochain/holochain-rust/pull/780)

## [0.0.3] - 2019-01-15
### Fixed
- build problems because of changes to upstream futures-preview crate
### Added
- Networking: beyond mock, using [n3h](https://github.com/holochain/n3h)
- Bridging now works and is configurable in the container (no capabilities yet) [#779](https://github.com/holochain/holochain-rust/pull/779) & [#776](https://github.com/holochain/holochain-rust/pull/776)
- Validation across network [#727](https://github.com/holochain/holochain-rust/pull/727)
- API/HDK:
    - CRUD for entries working
    - Node-to-node messaging [#746](https://github.com/holochain/holochain-rust/pull/746)
    - GetEntryOptions:
        - retrieve CRUD history & status
        - meta data: sources
    - GetLinksOptions
        - meta data: sources
    - GetLinks helpers: get_links_and_load
    - Query: return multiple entry types with glob matching [#781](https://github.com/holochain/holochain-rust/pull/781)
- Conductor:
    - configuration builder and config files
    - http interface [#823](https://github.com/holochain/holochain-rust/pull/823)
- hc command-line tool:
    - `run --persist` flag for keeping state across runs [#729](https://github.com/holochain/holochain-rust/pull/729/files)
    - Added env variables to activate real networking [#826](https://github.com/holochain/holochain-rust/pull/826)
- Groundwork for: capabilities & signals [#762](https://github.com/holochain/holochain-rust/pull/826) & [#732](https://github.com/holochain/holochain-rust/pull/732)
- Improved debug logging with log rules and colorization [#819](https://github.com/holochain/holochain-rust/pull/819)
- This change log!

### Changed
- API/HDK:
    - native return types (JsonStrings)
    - many places where we referred to "Hash" we now use the more correct term "Address"

## [0.0.2] - 2018-11-28
### Added
- mock networking
- `hc run` with support for
- multi-instance scenario testing
