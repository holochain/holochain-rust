# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.0.10-alpha1] - 2019-04-04

### Added

- Adds conductor handling of agent key creation in the context of DPKI [#1182](https://github.com/holochain/holochain-rust/pull/1182)
- Adds a `--path` option to `hc keygen` to specify the location of the generated keybundle. [#1194](https://github.com/holochain/holochain-rust/pull/1194)
- Adds pickle db for cas and eav [#1178](https://github.com/holochain/holochain-rust/pull/1178)
- Adds a `--quiet` option to `hc keygen` for machine-readable output, intended for use in scripts. [#1197](https://github.com/holochain/holochain-rust/pull/1197)
- Adds logging output for every failed WASM execution showing the call that caused this error. [#1200](https://github.com/holochain/holochain-rust/pull/1200) This helps with debugging "Arguement Deserialization failed" errors.
- Adds DNA hash to `hc package` output [#1212](https://github.com/holochain/holochain-rust/pull/1212)

### Changed

- `add_agent()` admin function now creates keystore file instead of just recording file in config [#1182](https://github.com/holochain/holochain-rust/pull/1182)
- One-time-signing now takes a vector of payloads, and returns a vector of signatures. [#1193](https://github.com/holochain/holochain-rust/pull/1193)
- Pins nixpkgs to Holo-Host channel in shell and CI [#1162](https://github.com/holochain/holochain-rust/pull/1162)

### Deprecated

### Removed

- Removes deprecated zome calling route [#1147](https://github.com/holochain/holochain-rust/pull/1147). This is a breaking change for users of hc-web-client prior to version 0.1.3.  Please upgrade to 0.1.3 or later and use the callZome syntax.
- Removes JsonString::From<String> and replaces it with JsonString::from_json(&str). This makes conversions more explicit and allows for validating that the string is actually valid json [#1184](https://github.com/holochain/holochain-rust/pull/1184)

### Fixed

-This pull request fixes the various issues with the pickledb implementation. Better guards and directory fixes [#1202]
(https://github.com/holochain/holochain-rust/pull/1202)

### Security

## [0.0.9-alpha] - 2019-03-31

### Added
- Adds hdk access to keystore [#1148](https://github.com/holochain/holochain-rust/pull/1148)

### Changed
- Performance optimization: don't recalculate DNA hash during handling of every network message but instead cache the DNA hash. [PR#1163](https://github.com/holochain/holochain-rust/pull/1163)

### Deprecated

### Removed

### Fixed

### Security


## [0.0.8-alpha] - 2019-03-21

### Added

- Adds Validation For CrudStatus as well as changes api for Crud and Link Validation Rules. [PR#1117] (https://github.com/holochain/holochain-rust/pull/1117)
- Adds `nix-shell` support for Mac OS X [#1132](https://github.com/holochain/holochain-rust/pull/1132)
- Adds `hc-test-all` command to `nix-shell` [#1132](https://github.com/holochain/holochain-rust/pull/1132)
- Adds `./scripts/nix/pod.sh` script to isolate/debug `nix-shell` commands [#1139](https://github.com/holochain/holochain-rust/pull/1139)
- Adds getting of Headers over the network [#1141](https://github.com/holochain/holochain-rust/pull/1141)
- Adds keystore and passphrase management service [#1104](https://github.com/holochain/holochain-rust/pull/1104)
- Adds tooling to manage dependencies in Cargo.toml [#1140](https://github.com/holochain/holochain-rust/pull/1140)
- Adds ability to enable logging via flag (`--logging`) to `hc run` command [#1151](https://github.com/holochain/holochain-rust/pull/1151)
- Adds `hc chain` command, which prints a raw text dump of a source chain [#1126](https://github.com/holochain/holochain-rust/pull/1126)


### Changed
- Conductor now waits for N3H to return p2p bindings [#1149](https://github.com/holochain/holochain-rust/pull/1149)
- `nix-shell` is now the recommended development approach on supported platforms [#1132](https://github.com/holochain/holochain-rust/pull/1132)
- Pins every dependant crate version with `=x.y.z` at the Cargo.toml level [#1140](https://github.com/holochain/holochain-rust/pull/1140)
- Breaking Change: `key_file` value now renamed to `keystore_file` in both config.toml files and the conductor's `admin/agent/add` interface [#1104](https://github.com/holochain/holochain-rust/pull/1104)
- EAVI adds are now optimized [#1166](https://github.com/holochain/holochain-rust/pull/1166)

### Deprecated

### Removed

- Removes all Cargo.lock files [#1140](https://github.com/holochain/holochain-rust/pull/1140)

### Fixed
- Adds Validation for Crud Reinstates EntryLifecycle. [PR#1143] (https://github.com/holochain/holochain-rust/pull/1143)
### Security

## [0.0.7-alpha] - 2019-03-19

### Added

- Adds the ability to pass in the token and provenance in zome calls for generating the capability request for the call. [PR#1077](https://github.com/holochain/holochain-rust/pull/1077)

### Changed

- Instantiate instance when creating through admin interface [#1067](https://github.com/holochain/holochain-rust/pull/1067)
- Use Content-type: application/json for remote signing service HTTP requests [#1067](https://github.com/holochain/holochain-rust/pull/1067)
- Check for duplicate IDs during integrity check [#1067](https://github.com/holochain/holochain-rust/pull/1067)

### Deprecated

### Removed

### Fixed
- Conductors running on Windows will be able to hit '/' route for UI server [PR#1128](https://github.com/holochain/holochain-rust/pull/1128)

### Security


## [0.0.6-alpha] - 2019-03-11

### Changed
- Replaces libzmq (zeromq) with websockets for ipc communication to networking module [#1055](https://github.com/holochain/holochain-rust/pull/1055)
- Changes `apt-get` dependencies installed for libsodium across linux OS [#1105](https://github.com/holochain/holochain-rust/pull/1105)

### Removed
- Removes bespoke `rust_sodium-sys` crate (using upstream now) [#1105](https://github.com/holochain/holochain-rust/pull/1105)

### Added
- New network setting via environment variable HC_N3H_LOG_LEVEL [#1085](https://github.com/holochain/holochain-rust/pull/1085)
- Ability to sign data via `hdk::sign` using the agent key [PR#1080](https://github.com/holochain/holochain-rust/pull/1080)
- Adds PUBLIC_TOKEN global variable for use in hdk::call in calling public functions. [PR#895](https://github.com/holochain/holochain-rust/pull/895)
- Adds an [ADR](doc/architecture/decisions/0017-capabilities.md) for capabilities [#895](https://github.com/holochain/holochain-rust/pull/895)
- CrudStatus works over network [#1048](https://github.com/holochain/holochain-rust/pull/1048)
- Adds utils submodule of hdk which contains the following helper functions [#1006](https://github.com/holochain/holochain-rust/pull/10006):
  - get_links_and_load_type - calls try_from for a given type when getting links
  - get_as_type - Similar but for a single entry
  - link_entries_bidir - Same as link_entries but creates link in both directions
  - commit_and_link - Save a line and commit and link in a single function
- Adds a `call` route to the json rpc for the conductor for making zome calls [PR#1090](https://github.com/holochain/holochain-rust/pull/1090).  Please note this route deprecates the `instance_id/zome/function` which will be removed in the future
- The `admin/dna/install_from_file` RPC method now takes an optional `expected_hash`, which performs an integrity check of the DNA file before installing it in the conductor [PR#1093](https://github.com/holochain/holochain-rust/pull/1093)
- Adds empty API function definitions to HDK that are only compiled for test targets to enable Rust native unit tests for Zomes [#989](https://github.com/holochain/holochain-rust/pull/989)
- Moves Crud Status tests to app_spec [#1096](https://github.com/holochain/holochain-rust/pull/1096)
- Adds cold build tests + support for debian and ubuntu xenial [#1105](https://github.com/holochain/holochain-rust/pull/1105)

### Fixed
- Validation of link entries gets retried now if base or target of the link were not yet accessible on the validating node. This fixes a bug where links have been invalid due to network timing issues [PR#1054](https://github.com/holochain/holochain-rust/pull/1054)
- Validation of any entry gets retried now if the validation package could not be retrieved from the source [PR#1059](https://github.com/holochain/holochain-rust/pull/1059)
- Scenario tests are more lenient to SyntaxError, TypeError, and other JS errors: buggy tests now merely fail rather than hanging indefinitely [#1091](https://github.com/holochain/holochain-rust/pull/1091)
- Fixes docker builds for `holochain/holochain-rust:develop` [#1107](https://github.com/holochain/holochain-rust/pull/1107)

## [0.0.5-alpha] - 2019-03-01

### Changed
- Relaxes Node JS version to 8.x in nix-shell [PR#955](https://github.com/holochain/holochain-rust/pull/955)
- Updates develop docker tag to use nix [PR#955](https://github.com/holochain/holochain-rust/pull/955)
- Updates bash script shebang to be nixos friendly [PR#955](https://github.com/holochain/holochain-rust/pull/955)
- Changes file name for cli packaging [PR#1036](https://github.com/holochain/holochain-rust/pull/1036)
  - `bundle.json` & `.hcpkg` unified to `YOUR_DNA_NAME.dna.json`
  - `.build` files renamed to `.hcbuild`
  - `hc package` now builds to `dist` directory by default, to match how `hc test` works

### Removed
- Removes legacy docker files [PR#955](https://github.com/holochain/holochain-rust/pull/955)

### Added
- Adds a panic handler to HDK-Rust and that reroutes infos about panics happening inside the WASM Ribosome to the instances logger [PR#1029](https://github.com/holochain/holochain-rust/pull/1029)
- Adds cmake and qt to mac os x install script [PR#955](https://github.com/holochain/holochain-rust/pull/955)
- Adds the current git-commit hash to the compile code of the core, and checks (with warning) for the same hash that was used to compile the wasm [PR#1050](https://github.com/holochain/holochain-rust/pull/1036)

## [0.0.4-alpha] - 2019-02-15

### Fixed
- Futures handling and zome function execution refactored which enables using complex API functions like `commit_entry` in callbacks such as `receive`.  This also fixes long standing flaky tests and blocking behaviors we have been experiencing. [#991](https://github.com/holochain/holochain-rust/pull/991)
### Changed
- Capabilities now separated from function declarations and renamed to `traits` in `define_zome!` and calling zome functions no longer uses capability name parameter [#997](https://github.com/holochain/holochain-rust/pull/997) & [#791](https://github.com/holochain/holochain-rust/pull/791)
- `hash` properties for `UiBundleConfiguration` and `DnaConfiguration` in Conductor config files is now optional [#966](https://github.com/holochain/holochain-rust/pull/966)
- `ChainHeader::sources()` is now `ChainHeader::provenances()` which stores both source address, and signature  [#932](https://github.com/holochain/holochain-rust/pull/932)
- `hdk::get_entry_results` supports return of ChainHeaders for all agents who have committed the same entry [#932](https://github.com/holochain/holochain-rust/pull/932)
- Renames the term Container and all references to it to Conductor [#942](https://github.com/holochain/holochain-rust/pull/942)
- Renames the `holochain_container` executable to simply `holochain` [#942](https://github.com/holochain/holochain-rust/pull/942)
- Renames the `cmd` crate (which implements the `hc` command line tool) to `cli` [#940](https://github.com/holochain/holochain-rust/pull/940)
- Encoded values in ribosome function's input/output are u64 (up from u32) [#915](https://github.com/holochain/holochain-rust/pull/915)
- Updated dependencies: [#924](https://github.com/holochain/holochain-rust/pull/924)
  * Rust nightly to `2019-01-24`
  * futures to `0.3.0-alpha.12`
- All chain headers are sent in the validation package, not just those for public entry types. [#926](https://github.com/holochain/holochain-rust/pull/926)
### Added
- Adds centralized documentation for environment variables in use by Holochain [#990](https://github.com/holochain/holochain-rust/pull/990)
- Adds command `hc keygen` which creates a new key pair, asks for a passphrase and writes an encrypted key bundle file to `~/.holochain/keys`. [#974](https://github.com/holochain/holochain-rust/pull/974)
- Adds an environment variable `NETWORKING_CONFIG_FILE` for specifing the location of the json file containing the network settings used by n3h. [#976](https://github.com/holochain/holochain-rust/pull/976)
- Adds an environment variable `HC_SIMPLE_LOGGER_MUTE` for use in testing which silences logging output so CI logs won't be too big. [#960](https://github.com/holochain/holochain-rust/pull/960)
- Adds Zome API function `hdk::sleep(std::time::Duration)` which works the same as `std::thread::sleep`.[#935](https://github.com/holochain/holochain-rust/pull/935)
- All structs/values to all HDK functions must implement `Into<JsonString>` and `TryFrom<JsonString>` (derive `DefaultJson` to do this automatically) [#854](https://github.com/holochain/holochain-rust/pull/854)
- HDK globals `AGENT_ADDRESS`, `AGENT_ID_STR`, `DNA_NAME` and `DNA_ADDRESS` are now set to real, correct values. [#796](https://github.com/holochain/holochain-rust/pull/796)
- `hc run` now looks for the --interface flag or `HC_INTERFACE` env var if you want to specify the `http` interface [#846]((https://github.com/holochain/holochain-rust/pull/846)
- NodeJS Conductor added to allow running conductors for testing purposes in JavaScript. [#1007](https://github.com/holochain/holochain-rust/pull/1007)
- Scenario API added to enable deterministic scenario tests for zome functions. See the [NodeJS Conductor README](nodejs_conductor/README.md) for details. [#942](https://github.com/holochain/holochain-rust/pull/942)
- `hdk::holochain_core_types::time::Iso8601` now supports validation and conversion to DateTime, and is sortable. [#917](https://github.com/holochain/holochain-rust/pull/917)
- `hdk::query_result` API supports return of ChainHeader and/or Entry data for the matched EntryType(s) [#868](https://github.com/holochain/holochain-rust/pull/868)
- Admin RPC functions added to container interface. Any (websocket) container interface that is configured with  `admin = true`  now can call a number of functions to remotely change any aspect of the container config. [#840](https://github.com/holochain/holochain-rust/pull/840)
- Adds a set of functions to the container RPC for managing static UI bundles and HTTP interfaces to these.  See rustdoc of `conductor_api::interface::ConductorApiBuilder` for a full description of these functions. [#919](https://github.com/holochain/holochain-rust/pull/919)
- Conductor can now serve static directories called ui_bundles over HTTP that can be configured in the container config toml file. This HTTP server also implements a virtual json file at "/_dna_connections.json" that returns the DNA interface (if any) the UI is configured to connect to. Hc-web-client will use this to automatically connect to the correct DNA interface on page load.  [#885](https://github.com/holochain/holochain-rust/pull/885)
- Adds Zome API function `hdk::remove_link(base,target,tag)` for removing links.  [#780](https://github.com/holochain/holochain-rust/pull/780)

## [0.0.3] - 2019-01-15
### Fixed
- build problems because of changes to upstream futures-preview crate [#864](https://github.com/holochain/holochain-rust/pull/864)
### Added
- Networking: beyond mock, using [n3h](https://github.com/holochain/n3h) [#831](https://github.com/holochain/holochain-rust/pull/831)
- Bridging now works and is configurable in the container (no capabilities yet) [#779](https://github.com/holochain/holochain-rust/pull/779) & [#776](https://github.com/holochain/holochain-rust/pull/776)
- Validation across network [#727](https://github.com/holochain/holochain-rust/pull/727)
- API/HDK: [#831](https://github.com/holochain/holochain-rust/pull/831)
    - CRUD for entries working
    - Node-to-node messaging [#746](https://github.com/holochain/holochain-rust/pull/746)
    - GetEntryOptions:
        - retrieve CRUD history & status
        - meta data: sources
    - GetLinksOptions
        - meta data: sources
    - GetLinks helpers: get_links_and_load
    - Query: return multiple entry types with glob matching [#781](https://github.com/holochain/holochain-rust/pull/781)
- Conductor: [#942](https://github.com/holochain/holochain-rust/pull/942)
    - configuration builder and config files
    - http interface [#823](https://github.com/holochain/holochain-rust/pull/823)
- hc command-line tool: [#831](https://github.com/holochain/holochain-rust/pull/831)
    - `run --persist` flag for keeping state across runs [#729](https://github.com/holochain/holochain-rust/pull/729/files)
    - Added env variables to activate real networking [#826](https://github.com/holochain/holochain-rust/pull/826)
- Groundwork for: capabilities & signals [#762](https://github.com/holochain/holochain-rust/pull/826) & [#732](https://github.com/holochain/holochain-rust/pull/732)
- Improved debug logging with log rules and colorization [#819](https://github.com/holochain/holochain-rust/pull/819)
- This change log! [#831](https://github.com/holochain/holochain-rust/pull/831)

### Changed
- API/HDK: [#831](https://github.com/holochain/holochain-rust/pull/831)
    - native return types (JsonStrings)
    - many places where we referred to "Hash" we now use the more correct term "Address"

## [0.0.2] - 2018-11-28
### Added
- mock networking [#831](https://github.com/holochain/holochain-rust/pull/831)
- `hc run` with support for [#831](https://github.com/holochain/holochain-rust/pull/831)
- multi-instance scenario testing [#831](https://github.com/holochain/holochain-rust/pull/831)
