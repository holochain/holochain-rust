# rust_sodium-sys - Change Log

## [0.10.3]
- update lazy_static to 1.2.0
- fix compilation with rustc 1.32.0

## [0.10.2]
- add missing trait for MSVC build

## [0.10.1]
- fix build: replace reqwest with http_req in build script
- use rust 1.29.0 stable, drop nightly
- use cargo fmt and cargo clippy

## [0.10.0]
- upgrade unwrap version to 1.2.0
- use rust 1.28.0 stable / 2018-07-07 nightly
- rustfmt 0.99.2 and clippy-0.0.212

## [0.9.0]
- upgrade libsodium version to 1.0.16
- replace 'use-installed-libsodium' feature with env var
- pull most outstanding sodiumoxide commits downstream
- add a hash check of downloaded libsodium files to build script
- use rust 1.25.0 stable

## [0.8.1]
- update URL for libsodium sources

## [0.8.0]
- Use rust 1.24.0 stable / 2018-02-05 nightly
- rustfmt 0.9.0 and clippy-0.0.186

## [0.7.2]
- Fix Android build error

## [0.7.1]
- Fixed issue causing libsodium to be built unoptimised on non-Windows platforms

## [0.7.0]
- Use rust 1.22.1 stable / 2017-11-23 nightly
- rustfmt 0.9.0 and clippy-0.0.174

## [0.6.0]
- Add support for iOS build targets

## [0.5.0]
- Use rust 1.19 stable / 2017-07-20 nightly
- rustfmt 0.9.0 and clippy-0.0.144
- Replace -Zno-trans with cargo check
- Make appveyor script using fixed version of stable

## [0.4.0]
- Changed build script for non-Windows platforms to only pass `--disable-pie` when a new env var is set.

## [0.3.1]
- Added fallback URL for Windows libsodium artefacts.

## [0.3.0]
- Ported several updates from sodiumoxide
- Upgraded libsodium version to 1.0.12
- Changed the default feature behaviour for rust_sodium-sys to download and unpack/build libsodium
- For Linux distros, only enable PIE for Ubuntu >= 15.04
- Added support for MSVC builds
- Changed to use rust 1.17 stable
- Updated CI script to run cargo_install from QA

## [0.2.0]
- Default to serde instead of rustc-serialize
- rustfmt 0.8.1
- enforce min powershell major version of 4 for compilation on Windows

## [0.1.2]
- Fix Windows build scripts by using curl.
- Fix ARM build by allowing the `trivial_casts` lint.
- Print build commands on failure.
- [Upstream pull - Make vector manipulation more efficient.](https://github.com/dnaq/sodiumoxide/commit/f509c90de1a5825abf67e1d8cd8cd70a35b91880)
- Added `init()` to every test.
- Updated dependencies.
- Added standard MaidSafe lint checks and fixed resulting warnings.

## [0.1.1]
- Bugfix for missed renaming of feature gate.

## [0.1.0]
- Initial fork from sodiumoxide including changes to build script.
- Added `init_with_rng()` to allow sodiumoxide_extras crate to be deprecated.
