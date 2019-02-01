# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- Encoded values in ribosome function's input/output are u64 (up from u32)
- Capabilities now separated from function declarations in `define_zome!` and calling zome functions no longer uses capability name parameter [#791](https://github.com/holochain/holochain-rust/pull/779)
- Updated dependencies:
  * Rust nightly to `2019-01-24`
  * futures to `0.3.0-alpha.12`
- Adjusted so that all chain headers are sent in the validation package, not just those for public entry types
### Added
- Added Zome API function `hdk::sleep(std::time::Duration)` which works the same as `std::thread::sleep`.
- All structs/values to all HDK functions must implement `Into<JsonString>` and `TryFrom<JsonString>` (derive `DefaultJson` to do this automatically)
- HDK globals `AGENT_ADDRESS`, `AGENT_ID_STR`, `DNA_NAME` and `DNA_ADDRESS` are now set to real, correct values.
- `hc run` now looks for the --interface flag or `HC_INTERFACE` env var if you want to specify the `http` interface [#846]((https://github.com/holochain/holochain-rust/pull/779)
- Scenario API added to enable deterministic scenario tests for zome functions. See the [NodeJS Container README](nodejs_container/README.md) for details.
- `hdk::query_result` API supports return of ChainHeader and/or Entry data for the matched EntryType(s)
- Admin RPC functions added to container interface. Any (websocket) container interface that is configured with
  `admin = true`  now can call the following functions to remotely change any aspect of the container config
  (intended to be used in an upcoming container admin UI):
  * `admin/dna/install_from_file` (install a DNA from a local file)
  * `admin/dna/uninstall`
  * `admin/dna/list`
  * `admin/instance/add`
  * `admin/instance/remove`
  * `admin/instance/start`
  * `admin/instance/stop`
  * `admin/instance/list` (list of all instances in config)
  * `admin/instance/running` (list of currently running instances)
  * `admin/interface/add` (starts the interface)
  * `admin/interface/remove` (stops the interface)
  * `admin/interface/add_instance` (restarts the interface to get change in effect)
  * `admin/interface/remove_instance` (restarts the interface to get change in effect)
  * `admin/interface/list`
  * `admin/agent/add`
  * `admin/agent/remove`
  * `admin/agent/list`
  * `admin/bridge/add`
  * `admin/bridge/remove`
  * `admin/bridge/list`
  
- Hosting of static files over HTTP to allow for container hosted web UIs
- UI bundle admin RPC functions
   Adds a further set of functions to the container RPC for managing 
   static UI bundles and HTTP interfaces to these.
   This adds the following RPC endpoints:
   
   * `admin/ui/install`
   * `admin/ui/uninstall`
   * `admin/ui/list`
   * `admin/ui_interface/add`
   * `admin/ui_interface/remove`
   * `admin/ui_interface/list`
   * `admin/ui_interface/start`
   * `admin/ui_interface/stop`

  See rustdoc of `container_api::interface::ContainerApiBuilder` for a full description of these functions.
- Container can serve static directories called ui_bundles over HTTP that can be configured in the container config toml file. This HTTP server also implements a virtual json file at "/_dna_connections.json" that returns the DNA interface (if any) the UI is configured to connect to. Hc-web-client will use this to automatically connect to the correct DNA interface on page load.

### Removed

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
- Container:
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
