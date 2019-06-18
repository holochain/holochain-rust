{ holonix, pkgs, release, pulse }:
let
  name = "hc-release-pulse-notes";

  template =
  ''
  # ${release.version.current} release {{ release-date }}
  {{ pulse-notes }}
  See the [Dev Pulse](${pulse.url}) & [change log](https://github.com/holochain/holochain-rust/blob/release-${release.version.current}/CHANGELOG.md) for complete details.
  ## **Installation**
  This release consists of binary builds of:
  - the [`hc` development command-line tool](https://github.com/holochain/holochain-rust/blob/${release.tag}/cli/README.md)
  - [`holochain` deployment conductor](https://github.com/holochain/holochain-rust/blob/${release.tag}/conductor/README.md) for different platforms.
  To install, simply download and extract the binary for your platform.
  See our [installation quick-start instructions](https://developer.holochain.org/start.html) for details.
  Rust and NodeJS are both required for `hc` to build and test DNA:
  - [Rust](https://www.rust-lang.org/en-US/install.html)
  - Must be `${holonix.rust.nightly.version}` build with the WASM build target.
    Once you have first installed rustup:
    ```
    rustup toolchain install ${holonix.rust.nightly.version}
    rustup default ${holonix.rust.nightly.version}
    rustup target add wasm32-unknown-unknown --toolchain ${holonix.rust.nightly.version}
    ```
  - [Node.js](https://nodejs.org) version 8 or higher
  - E2E tests for Holochain apps are written in Javascript client-side and executed in NodeJS through websockets
  - For further info, check out [the holochain-nodejs module](https://www.npmjs.com/package/@holochain/holochain-nodejs)
  ### **Which Binary?**
  Download only the binaries for your operating system.
  - MacOS: `cli-${release.tag}-x86_64-apple-darwin.tar.gz`
  - Linux: `cli-${release.tag}-x86_64-ubuntu-linux-gnu.tar.gz`
  - Windows:
  - mingw build system: `cli-${release.tag}-x86_64-pc-windows-gnu.tar.gz`
  - Visual Studio build system: `cli-${release.tag}-x86_64-pc-windows-msvc.tar.gz`
  All binaries are for 64-bit operating systems.
  32-bit systems are NOT supported.
  '';

  script = pkgs.writeShellScriptBin name
  ''
  TEMPLATE=$( echo '${template}' )
  DATE_PLACEHOLDER='{{ release-date }}'
  DATE=$( date --iso -u )
  WITH_DATE=''${TEMPLATE/$DATE_PLACEHOLDER/$DATE}
  PULSE_PLACEHOLDER='{{ pulse-notes }}'
  # magic
  # gets a markdown version of pulse
  # greps for everything from summary to details (not including details heading)
  # deletes null characters that throw warnings in bash
  PULSE_NOTES=$( curl -s https://md.unmediumed.com/${pulse.url} | grep -Pzo "(?s)(###\s+\**Summary.*)(?=###\s+\**Details)" | tr -d '\0' )
  WITH_NOTES=''${WITH_DATE/$PULSE_PLACEHOLDER/$PULSE_NOTES}
  echo "$WITH_NOTES"
  '';
in
{
 buildInputs = [ script ];
}
