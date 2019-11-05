let
 release-commit = "796cf81bce9ef5968241d7a716936a52be2dbd4e";
 current = "0.0.37-alpha2";
 previous = "0.0.37-alpha1";
 # tag will ultimately be current version when it hits holonix
 # https://github.com/holochain/holonix/blob/master/release/default.nix#L7
 tag = "v${current}";
 holonix-version = "v0.0.37";
 holonix-sha256 = "1dhv61x6lvpxvrs6ij44piqswb62hgn0q9fdxv7fnhc1a9grqcr3";
in
rec {

 # configure holonix itself
 holonix = {

  # true = use a github repository as the holonix base (recommended)
  # false = use a local copy of holonix (useful for debugging)
  use-github = true;

  # configure the remote holonix github when use-github = true
  github = {

   # can be any github ref
   # branch, tag, commit, etc.
   ref = holonix-version;

   # the sha of what is downloaded from the above ref
   # note: even if you change the above ref it will not be redownloaded until
   #       the sha here changes (the sha is the cache key for downloads)
   # note: to get a new sha, get nix to try and download a bad sha
   #       it will complain and tell you the right sha
   sha256 = holonix-sha256;

   # the github owner of the holonix repo
   owner = "holochain";

   # the name of the holonix repo
   repo = "holonix";
  };

  # configuration for when use-github = false
  local = {
   # the path to the local holonix copy
   path = ../holonix;
  };

 };

 release = {
  hook = {
   # sanity checks before deploying
   # to stop the release
   # exit 1
   preflight = ''
hc-release-audit
hn-release-hook-preflight-manual
'';

   # bump versions in the repo
   version = ''
hn-release-hook-version-rust
hn-release-hook-version-readme
hc-cli-release-hook-version
hc-release-hook-version
# refresh root Cargo.lock file
echo "updating cargo"
cargo update
'';

   # publish artifacts to the world
   publish = ''
echo "go look at travis for binary building!"
hc-release-hook-publish
'';
  };

  # the commit hash that the release process should target
  # this will always be behind what ends up being deployed
  # the release process needs to add some commits for changelog etc.
  commit = release-commit;

  # the semver for prev and current releases
  # the previous version will be scanned/bumped by release scripts
  # the current version is what the release scripts bump *to*
  version = {
   current = current;
   # not used by version hooks in this repo
   previous = previous;
  };

  github = {
   # markdown to inject into github releases
   # there is some basic string substitution {{ xxx }}
   # - {{ changelog }} will inject the changelog as at the target commit
   template = ''
   {{ changelog }}

   # Installation

   This release includes binaries of:

   - the [`hc` development command-line tool](https://github.com/holochain/holochain-rust/blob/${tag}/cli/README.md)
   - [`holochain` deployment conductor](https://github.com/holochain/holochain-rust/blob/${tag}/conductor/README.md) for different platforms.

   ## Very much recommended install

   The recommended installation process is to follow the [developer quick-start guide](https://developer.holochain.org/start.html).

   The approach in the quick start guide:

   - provides additional supporting tools like rust & node
   - shows you how to keep up to date with the latest versions of everything
   - makes minimal changes to your machine overall
   - is relatively difficult to screw up

   ## Bothersome manual install

   **IMPORTANT:** Manual holochain installations can conflict with the installer.

   Either binary is installed by being placed anywhere on your `$PATH`.
   This is different for everyone and depends how your machine is configured.

   For `hc` to build and test DNA Rust and NodeJS are both needed.

   ### Which Rust?

   The binaries for this release were built with rust from holonix version ${holonix-version}.
   WASM needs the `wasm32-unknown-unknown` rust target on your toolchain.

   ### Which NodeJS?

   Node is used to run end to end tests as a client of the holochain.
   Holochain exposes websockets for node to interact with.

   We recommend nodejs 10+.

   ### Which Binary?

   Download the binaries for your operating system.

   - MacOS: `cli-${tag}-x86_64-apple-darwin.tar.gz`
   - Linux: `cli-${tag}-x86_64-generic-linux-gnu.tar.gz`
   - Windows:
     - Visual Studio build system (default): `cli-${tag}-x86_64-pc-windows-msvc.tar.gz`
     - mingw build system: `cli-${tag}-x86_64-pc-windows-gnu.tar.gz`

   All binaries are for 64-bit operating systems.
   32-bit systems are NOT supported.
'';

   # owner of the github repository that release are deployed to
   owner = "holochain";

   # repository name on github that release are deployed to
   repo = "holochain-rust";

   # canonical local upstream name as per `git remote -v`
   upstream = "origin";

  };

  # non-standard, overridden by holonix internally anyway
  # used by check artifacts
  tag = tag;
 };
}
