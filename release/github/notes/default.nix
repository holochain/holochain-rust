{ holonix, pkgs, release }:
let
  name = "hc-release-github-notes";

  heading-placeholder = "{{ version-heading }}";
  changelog-placeholder = "{{ changelog }}";

  template =
  ''
  {{ changelog }}

  # Installation

  This release includes binaries of:

  - the [`hc` development command-line tool](https://github.com/holochain/holochain-rust/blob/${release.tag}/cli/README.md)
  - [`holochain` deployment conductor](https://github.com/holochain/holochain-rust/blob/${release.tag}/conductor/README.md) for different platforms.

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

  The binaries for this release were built with rust version `${holonix.rust.nightly.version}`.
  WASM needs the `wasm32-unknown-unknown` rust target on your toolchain.

  ### Which NodeJS?

  Node is used to run end to end tests as a client of the holochain.
  Holochain exposes websockets for node to interact with.

  We recommend nodejs 10+.

  ### Which Binary?

  Download the binaries for your operating system.

  - MacOS: `cli-${release.tag}-x86_64-apple-darwin.tar.gz`
  - Linux: `cli-${release.tag}-x86_64-generic-linux-gnu.tar.gz`
  - Windows:
    - Visual Studio build system (default): `cli-${release.tag}-x86_64-pc-windows-msvc.tar.gz`
    - mingw build system: `cli-${release.tag}-x86_64-pc-windows-gnu.tar.gz`

  All binaries are for 64-bit operating systems.
  32-bit systems are NOT supported.
  '';

  script = pkgs.writeShellScriptBin name
  ''
  changelog=$( git show ${release.commit}:./CHANGELOG-UNRELEASED.md )
  heading_placeholder="${heading-placeholder}"
  heading="## [${release.version.current}] - $(date --iso --u)"
  changelog=''${changelog/$heading_placeholder/$heading}

  template=$( echo '${template}' )
  changelog_placeholder="${changelog-placeholder}"
  output=''${template/$changelog_placeholder/$changelog}
  echo "''${output}"
  '';
in
{
 buildInputs = [ script ];
}
