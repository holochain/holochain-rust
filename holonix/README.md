# Holonix

Comprehensive Holochain Core ops tooling (build/test/release) for use in nix compatible environments (NixOS and anywhere `nix-shell` runs).

## Scope

Where `nix-shell` is supported Holonix is the standard approach for:

- Installing dependencies
- Running tests
- Building wasm
- Building binaries
- Implementing CI scripts
- Conductor management
- Installing situational tooling (e.g. `cargo-edit` for versioning)
- Release management, automation and deploying official binaries
- BAU automation and reporting tasks**
- Downloading prebuilt official binaries from github**
- Managing and updating binaries across official releases**

** Coming Soon!

### Supported environments

NixOS is a dedicated linux distro and so is standalone by its nature.

`nix-shell` is a cli tool supported across Mac OS and Linux distros.

https://nixos.org/nix/download.html

There is theoretical support for `nix-shell` in Windows Subsystem Linux (WSL).
WSL support is currently broken upstream https://github.com/NixOS/nix/issues/1203

**Use `nix-shell` if you are on anything other than Windows!**

### Why nix?

Nix approach offers unique benefits:

- Dependencies are injected into a single shell session only
  - Minimal modifications to the host environment
  - No need to maintain/rerun/troubleshoot installation scripts
  - Further isolation from host environment can be achieved with `nix-shell --pure`
- Dependencies are hashed
  - "Dependency hell" is avoided
  - Nice parallels with Holochain's hashed zomes model
  - Security + reliability benefits
- Dependencies can be garbage collected with the `nix-collect-garbage` command
- Single "package manager" across most operating systems
- Ability to ship utility scripts in the `shell.nix` file
- Access to the nix functional programming language for dependencies/script management
  - Allows for a structured approach to coding the ops infrastructure
  - Makes modern programming paradigm such as immutability, scopes, data structures, etc. in the ops tooling as opposed limitations and lack of expressivity of Bash/Makefile/etc.
- NixOS runs on HoloPorts so `nix-shell` provides similar behaviour/environment
  - Everything is pinned to the same nix channel and commit

And other benefits:

- Active and helpful community of contributors
  - IRC at #nixos
  - wiki at https://nixos.wiki/
- Ability to bundle scripts into convenient binaries for daily tasks
- Nice CLI interface with many flexible, well documented configuration options
- Available in shebang form as `#! /usr/bin/env nix-shell`
  - http://iam.travishartwell.net/2015/06/17/nix-shell-shebang/

## Installation

Follow the instructions to install `nix-shell` from:

https://nixos.org/nix/download.html

Once installed run `nix-shell` from the `holochain-rust` repository root.
The `nix-shell` command will detect the `shell.nix` file and build everything.

The first run will be slow as nix downloads, builds and caches dependencies.
Everything is downloaded to `/nix/store/...` and re-used on subsequent runs.

## Contributing

The structure of our nix derivations is designed to be as modular as possible.

The folder structure is a basic heirarchy, something like:

```
holonix
 |_github
 |  |_config.nix
 |_nixpkgs
 | |_nixpkgs.nix
 |_node
 | |_src
 | | |_flux.nix
 | | |_...
 | |_build.nix
 |_...
shell.nix
```

The `shell.nix` file is used by `nix-shell` automatically by default.
This consumes `holonix/**` and does not provide any new derivations.
End-users should not need to interact with `holonix/**` outside `shell.nix`.

There are a few basic conventions to follow:

- Nest folders according to theme/tech/specificity
  - e.g. conductor management for conductor `x` sits under `conductor/x/**`
- All configuration strings and other primitives sit in a local `config.nix`
- Structure configuration as nested `foo.bar` rather than `foo-bar`
  - e.g. `holonix/release/config.nix` has a few good examples of this
- All used and generated inputs to build the nix derivations sit in a local `build.nix`
  - `build.nix` files should "bubble up" to the root one level at a time
    - e.g. `build.nix` imports `conductor/build.nix` imports `conductor/node/build.nix`
  - root `build.nix` and `src` should only aggregate deeper derivations
- Scripts for binaries sit in named `foo.nix` files under `thing/src/foo.nix`
  - There is standard boilerplate for this, see an existing file for examples
  - Use `pkgs.writeShellScriptBin` by default
  - derived nix CLI commands are named following the path sans `src`
    - e.g. `holonix/foo/bar/src/baz.nix` becomes `hc-foo-bar-baz`
- Make liberal use of `let .. in ..` scoping constructs in `.nix` files
- Put functions for builds in `lib.nix` files
  - e.g. `holonix/dist/rust/src/lib.nix`
- File names can stutter but command names should not
  - e.g. `holonix/rust/fmt/src/fmt.nix` for `hc-rust-fmt`
- Use `install.nix` for scripts that install things outside of what nix manages
  - Try to minimise use of additional install scripts, nix should handle as
    much as possible.
  - e.g. cargo installs things to the user's home directory
