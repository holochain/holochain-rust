# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

{{ version-heading }}

### Added

* Adds EncryptedSeed and seed.encrypt() allow for easy passphrase encrypting/decrypting of any of the existing seed types. Adds the MnemonicableSeed trait allows seeds to be converted to/from BIP39 mnemonics. [#1687](https://github.com/holochain/holochain-rust/pull/1687) 
* added nix for `hc-conductor-install` and `hc-conductor-uninstall` based on `cargo`
* When loading a hand-written or generated conductor config containing a TestAgent (`test_agent = true`), rewrite the config file so that the test agent's `public_address` is correct, rather than the arbitrary value that was specified before the `public_address` was actually known. [#1692](https://github.com/holochain/holochain-rust/pull/1692)

### Changed

### Deprecated

### Removed

### Fixed

### Security
