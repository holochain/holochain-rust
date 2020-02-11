# Holochain Core Development Environment Setup

## Installation & Usage

**Important:** the instructions in this readme are for developers intending work on Holochain code-base itself, not Holochain application developers.  If you want to use Holochain, please proceed to the instructions on the quick start installation guide: **https://developer.holochain.org/start.html**

**The following instructions are for developing Holochain Core or the HDK itself**

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

[The Holonix documentation](https://docs.holochain.love/) has more information.

#### Troubleshooting

Default `nix-shell` behaviour preserves some of the user's environment, simply
_adding_ to it rather than _isolating_ from it.

This can cause problems if your user has cruft that conflicts with what nix is
doing, e.g. existing `cargo` or `npm` installations/environment variables.

If you are seeing an issue in `nix-shell` that others are not seeing, try using
our isolation script `./scripts/nix/pod.sh` to debug the command.

For example:

```shell
./nix/pod.sh 'hc-rust-test'
```

or even:

```shell
./nix/pod.sh hc-test
```

#### Future deployments

In the future we plan to distribute binaries through nixpkgs.
This would to enable the following:

```shell
# doesn't work yet... watch this space!
nix-shell -p holochain --run holochain ...
```

### Building for Android
Note there is an article written on how to build Holochain for Android, read it [here](doc/holochain_101/src/building_for_android.md).
