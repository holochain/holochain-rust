# Holochain Core Development Environment Setup

## Installation & Usage

**Important:** the instructions in this readme are for developers intending work on Holochain code-base itself, not Holochain application developers.  If you want to use Holochain, please proceed to the instructions on the quick start installation guide: **https://developer.holochain.org/start.html**

**The following instructions are for developing Holochain Core or the HDK itself**

There are two components needed currently to run Holochain applications, the core (what's in this repo) and also [the networking engine](https://github.com/holochain/n3h).  You can install and work on core using the built-in mock network following the instructions below, but if you want to actually test out your apps using the real networking, you will have to install [the networking component](https://github.com/holochain/n3h) following the instructions in the README there.  (Note: please see the instructions in the guide book for [`hc`](https://developer.holochain.org/guide/latest/hc_configuring_networking.html) or the [production Conductor](https://developer.holochain.org/guide/latest/conductor_networking.html) for how to configure the tools to use and activate the networking component.

There are three approaches to building and testing Holochain: using `nix-shell`, `make`, `docker`:

### Nix Shell (Supported: Ubuntu, Debian, Mac OS X, Vagrant & Nix OS)

The `nix-shell` command from the nixos team is the preferred way to work with Holochain.

NixOS is an entire operating system but the `nix-shell` is simply a tool to manage dependencies for an individual shell session.

To install `nix-shell`:

```shell
# basic deps needed on ubuntu/debian
apt-get update && apt-get install -y curl bzip2

# this installs on all (non-windows) systems
curl https://nixos.org/nix/install | sh
```

Follow any further instructions output to the terminal during installation.

[The website](https://nixos.org/nix/download.html) has more details.

Running the `nix-shell` command from inside the root of the repository will detect and use the `shell.nix` file.

The `nix-shell` approach offers unique benefits:

- Dependencies are injected into a single shell session only
  - Minimal modifications to the host environment
  - No need to maintain/rerun/troubleshoot installation scripts
  - Further isolation from host environment can be achieved with `nix-shell --pure`
- Dependencies are hashed
  - "Dependency hell" is avoided
  - Nice parallels with the hashed zomes model
  - Security + reliability benefits
- Dependencies can be garbage collected with the `nix-collect-garbage` command
- Single "package manager" across most operating systems
- Ability to ship utility scripts in the `shell.nix` file
- Access to the nix functional programming language for dependencies/script management
- NixOS runs on HoloPorts so `nix-shell` provides similar behaviour/environment

If you have a nix friendly system, this is the fastest and most reliable way to develop and test.

We also offer `nix-shell` support virtualised through [Vagrant](https://www.vagrantup.com/).

This is the easiest way to use `nix-shell` on Windows (docker requires a Windows Pro/Enterprise/Education license).

First install [Vagrant](https://www.vagrantup.com/) and [VirtualBox](https://www.virtualbox.org/).

```
cd /path/to/holochain-rust
vagrant plugin install vagrant-disksize
vagrant up
vagrant ssh
cd /vagrant
nix-shell
```

Once in a `nix-shell` you can run `hc -h` to see available subcommands.

#### Troubleshooting

Default `nix-shell` behaviour preserves some of the user's environment, simply
_adding_ to it rather than _isolating_ from it.

This can cause problems if your user has cruft that conflicts with what nix is
doing, e.g. existing `cargo` or `npm` installations/environment variables.

If you are seeing an issue in `nix-shell` that others are not seeing, try using
our isolation script `./scripts/nix/pod.sh` to debug the command.

For example:

```shell
./scripts/nix/pod.sh 'hc-rust-test'
```

or even:

```shell
./scripts/nix/pod.sh hc-test
```

#### Future deployments

In the future we plan to distribute binaries through nixpkgs.
This would to enable the following:

```shell
# doesn't work yet... watch this space!
nix-shell -p holochain --run holochain ...
```

### Make (Supported: Ubuntu, Debian & Mac OS X)

For Linux/OSX you can install the prerequisites directly into the host environment with:

``` shell
cd path/to/holochain
./scripts/install/auto.sh
```

**Note**: the script will install [homebrew](https://brew.sh/) on mac os x
**Note**: the script will install dependencies with `apt-get` on linux

After the install script completes successfully, you can start local development using `make`

Running the `make` command will:

1. install (or update to) the correct version of rust
2. build all the rust libraries from the source code in this repository.
3. build and install the command-line tools.

**Note**: it's very important to use the rust version specified in the Makefile! Since we are using nightly rust builds, the language is changing rapidly and sometimes introduces breaking changes that we haven't adapted to yet. Don't just use the latest nightly.

**Note**: The installation script evolves over time alongside core.
The installation script is idempotent.
Rerun the script after each upgrade/downgrade.

### Docker (Supported: Ubuntu, Debian, Mac OS X, Nix OS, Windows)

We support [docker](https://www.docker.com/).
The `docker` folder contains scripts to build and run docker images.

The `holochain/holochain-rust:latest` docker image is an alpine NixOS rebuilt nightly.
The build process warms nix and incrementally compiles cargo/wasm/neon for faster feedback.

### Windows

You will need to install rust manually.

Rustup `https://rustup.rs/#` is likely the best option.

The rust language moves very fast on the nightly channel.

It is very important to be using the correct nightly version.

Currently this is:

`nightly-2019-07-14-x86_64-pc-windows-msvc`

The nightly version we test/develop against can always be found in the .travis.yml file.

#### Running tests

Run:

```shell
. docker/run-test
```
or

``` shell
make test
```

or

``` shell
nix-shell --run hc-rust-test
```

Note that there are also make commands for running the tests of just core, or the command-line line tools or app_spec separately:

``` shell
make test_cli
make test_holochain
make test_app_spec
make build_nodejs_conductor
```

### Building for Android
Note there is an article written on how to build Holochain for Android, read it [here](doc/holochain_101/src/building_for_android.md).

## Upgrading

Upgrading to a new tagged release of Holochain may include new/changed system dependencies.

__If not using `nix-shell` we strongly recommend rerunning `./scripts/install/auto.sh` when upgrading core.__

The script is designed to be idempotent. This means there is no harm re-running it.
