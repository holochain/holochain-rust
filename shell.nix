let

  # https://vaibhavsagar.com/blog/2018/05/27/quick-easy-nixpkgs-pinning/
  inherit (import <nixpkgs> {}) fetchgit;
  # nixos holo-host channel @ 2019-04-02
  channel-holo-host = fetchgit {
    url = "https://github.com/Holo-Host/nixpkgs-channels.git";
    rev = "680f9d7ea90dd0b48b51f29899c3110196b0e913";
    sha256 = "07glx6r08l8hwzh8xzj8i0hj6ak42iswqfb9hbhs75rqq56zq43a";
  };

  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);

  pkgs = import channel-holo-host {
    overlays = [ moz_overlay ];
  };

  # https://stackoverflow.com/questions/51161225/how-can-i-make-macos-frameworks-available-to-clang-in-a-nix-environment
  frameworks = if pkgs.stdenv.isDarwin then pkgs.darwin.apple_sdk.frameworks else {};

  date = "2019-01-24";
  wasmTarget = "wasm32-unknown-unknown";

  rust-build = (pkgs.rustChannelOfTargets "nightly" date [ wasmTarget ]);

  hc-node-flush = pkgs.writeShellScriptBin "hc-node-flush"
  ''
  echo "flushing node artifacts"
  find . -wholename "**/node_modules" | xargs -I {} rm -rf  {};
  find . -wholename "./nodejs_conductor/bin-package" | xargs -I {} rm -rf {};
  find . -wholename "./nodejs_conductor/build" | xargs -I {} rm -rf {};
  find . -wholename "./nodejs_conductor/dist" | xargs -I {} rm -rf {};
  '';

  hc-cargo-flush = pkgs.writeShellScriptBin "hc-cargo-flush"
  ''
   echo "flushing cargo"
   rm -rf ~/.cargo/registry;
   rm -rf ~/.cargo/git;
   find . -wholename "**/.cargo" | xargs -I {} rm -rf {};
   find . -wholename "**/target" | xargs -I {} rm -rf {};
  '';
  hc-cargo-lock-flush = pkgs.writeShellScriptBin "hc-cargo-lock-flush"
  ''
  echo "flushing cargo locks"
  find . -name "Cargo.lock" | xargs -I {} rm {};
  '';

  hc-flush-all = pkgs.writeShellScriptBin "hc-flush-all"
  ''
  hc-node-flush
  hc-cargo-flush
  hc-cargo-lock-flush
  '';

  hc-install-node-conductor = pkgs.writeShellScriptBin "hc-install-node-conductor"
  ''
  hc-node-flush;
   . ./scripts/build_nodejs_conductor.sh;
  '';

  hc-install-tarpaulin = pkgs.writeShellScriptBin "hc-install-tarpaulin"
  ''
   if ! cargo --list | grep --quiet tarpaulin;
   then
    RUSTFLAGS="--cfg procmacro2_semver_exempt" cargo install cargo-tarpaulin;
   fi;
  '';
  hc-tarpaulin = pkgs.writeShellScriptBin "hc-tarpaulin" "cargo tarpaulin --ignore-tests --timeout 600 --all --out Xml --skip-clean -v -e holochain_core_api_c_binding -e hdk -e hc -e holochain_core_types_derive";

  hc-install-fmt = pkgs.writeShellScriptBin "hc-install-fmt"
  ''
   rustup component add rustfmt
  '';

  hc-install-edit = pkgs.writeShellScriptBin "hc-install-edit"
  ''
   cargo install cargo-edit
  '';
  hc-cargo-toml-set-ver = pkgs.writeShellScriptBin "hc-cargo-toml-set-ver"
  ''
   # node dist can mess with the process
   hc-node-flush
   find . -name "Cargo.toml" | xargs -I {} cargo upgrade "$1" --all --manifest-path {}
  '';
  hc-cargo-toml-grep-unpinned = pkgs.writeShellScriptBin "hc-cargo-toml-grep-unpinned"
  ''
   find . -type f \( -name "Cargo.toml" -or -name "Cargo.template.toml" \) \
     | xargs cat \
     | grep -Ev '=[0-9]+\.[0-9]+\.[0-9]+' \
     | grep -E '[0-9]+' \
     | grep -Ev '(version|edition|codegen-units)' \
     | cat
  '';
  hc-cargo-toml-test-ver = pkgs.writeShellScriptBin "hc-cargo-toml-test-ver"
  ''
   # node dists can mess with the process
   hc-node-flush

   # loop over all tomls
   # find all possible upgrades
   # ignore upgrades that are just unpinning themselves (=x.y.z will suggest x.y.z)
   # | grep -vE 'v=([0-9]+\.[0-9]+\.[0-9]+) -> v\1'
   echo "attempting to suggest new pinnable crate versions"
   find . -name "Cargo.toml" \
     | xargs -P "$NIX_BUILD_CORES" -I {} cargo upgrade --dry-run --allow-prerelease --all --manifest-path {} \
     | grep -vE 'v=[0-9]+\.[0-9]+\.[0-9]+'

   hc-cargo-toml-grep-unpinned
  '';

  hc-install-cli = pkgs.writeShellScriptBin "hc-install-cli" "cargo build -p hc --release && cargo install -f --path cli";
  hc-install-conductor = pkgs.writeShellScriptBin "hc-install-conductor" "cargo build -p holochain --release && cargo install -f --path conductor";

  hc-test-cli = pkgs.writeShellScriptBin "hc-test-cli" "cd cli && cargo test";
  hc-test-app-spec = pkgs.writeShellScriptBin "hc-test-app-spec" "cd app_spec && . build_and_test.sh";
  hc-test-node-conductor = pkgs.writeShellScriptBin "hc-test-node-conductor" "cd nodejs_conductor && npm test";

  hc-fmt = pkgs.writeShellScriptBin "hc-fmt" "cargo fmt";
  hc-fmt-check = pkgs.writeShellScriptBin "hc-fmt-check" "cargo fmt -- --check";

  # runs all standard tests and reports code coverage
  hc-codecov = pkgs.writeShellScriptBin "hc-codecov"
  ''
   hc-install-tarpaulin && \
   hc-tarpaulin && \
   bash <(curl -s https://codecov.io/bash);
  '';


  # simulates all supported ci tests in a local circle ci environment
  ci = pkgs.writeShellScriptBin "ci"
  ''
   circleci-cli local execute
  '';

  build-wasm = wasm-path:
  ''
   export WASM_PATH=${wasm-path}/
   cargo build --release --target wasm32-unknown-unknown --manifest-path "$WASM_PATH"Cargo.toml --target-dir "$HC_TARGET_PREFIX""$WASM_PATH"target;
  '';
  wasm-paths = [
   "hdk-rust/wasm-test"
   "wasm_utils/wasm-test/integration-test"
   "conductor_api/wasm-test"
   "conductor_api/test-bridge-caller"
   "core/src/nucleus/actions/wasm-test"
  ];
  hc-build-wasm = pkgs.writeShellScriptBin "hc-build-wasm"
  ''
   ${pkgs.lib.concatMapStrings (path: build-wasm path) wasm-paths}
  '';
  hc-test = pkgs.writeShellScriptBin "hc-test"
  ''
   hc-build-wasm
   HC_SIMPLE_LOGGER_MUTE=1 cargo test --all --release --target-dir "$HC_TARGET_PREFIX"target;
  '';

  hc-test-all = pkgs.writeShellScriptBin "hc-test-all"
  ''
   hc-fmt-check \
   && hc-build-wasm \
   && hc-install-cli \
   && hc-install-conductor \
   && hc-install-node-conductor \
   && hc-test-app-spec
  '';

  release-process-url = "https://hackmd.io/6HH-O1KuQ7uYzKrsI6ooIw";
  repo = "holochain/holochain-rust";
  upstream = "origin";
  # the unique hash at the end of the medium post url
  # e.g. https://medium.com/@holochain/foos-and-bars-4867d777de94
  # would be 4867d777de94
  pulse-url-hash = "4867d777de94";
  pulse-version = "22";
  pulse-commit = "0a524d3be580249d54cf5073591fa9fe1f30a174";
  core-previous-version = "0.0.8-alpha";
  core-version = "0.0.9-alpha";
  node-conductor-previous-version = "0.4.7-alpha";
  node-conductor-version = "0.4.8-alpha";

  pulse-tag = "dev-pulse-${pulse-version}";
  hc-prepare-pulse-tag = pkgs.writeShellScriptBin "hc-prepare-pulse-tag"
  ''
  echo
  echo 'tagging commit for pulse version ${pulse-version}'
  echo
  git fetch --tags
  if git tag | grep -q "${pulse-tag}"
   then
    echo "pulse tag for pulse ${pulse-version} already exists locally! doing nothing..."
    echo "pulse commit: $(git show-ref -s ${pulse-tag})"
    echo "to push upstream run: git push ${upstream} ${pulse-tag}"
   else
    echo "tagging..."
    git tag -a ${pulse-tag} ${pulse-commit} -m 'Dev pulse ${pulse-version}'
    echo "pushing..."
    git push ${upstream} ${pulse-tag}
    echo $'pulse tag ${pulse-tag} created and pushed'
  fi
  echo
  echo 'pulse tag on github: https://github.com/holochain/holochain-rust/releases/tag/${pulse-tag}'
  echo
  '';

  release-branch = "release-${core-version}";
  hc-prepare-release-branch = pkgs.writeShellScriptBin "hc-prepare-release-branch"
  ''
   echo
   echo 'preparing release branch'
   echo

   git fetch
   if git tag | grep -q "${release-branch}"
   then
    echo "There is a tag with the same name as the release branch ${release-branch}! aborting..."
    exit 1
   fi

   echo
   echo 'checkout or create release branch'
   if git branch | grep -q "${release-branch}"
    then
     git checkout ${release-branch}
     git pull
    else
     git checkout ${pulse-commit}
     git checkout -b ${release-branch}
     git push -u ${upstream} ${release-branch}
   fi
   echo
  '';

  hc-prepare-crate-versions = pkgs.writeShellScriptBin "hc-prepare-crate-versions"
  ''
   echo "bumping core version from ${core-previous-version} to ${core-version} in Cargo.toml"
   find . \
    -name "Cargo.toml" \
    -not -path "**/.git/**" \
    -not -path "**/.cargo/**" \
    -not -path "./nodejs_conductor*" \
    | xargs -I {} \
    sed -i 's/^\s*version\s*=\s*"${core-previous-version}"\s*$/version = "${core-version}"/g' {}

   echo "bumping core versions from ${core-previous-version} to ${core-version} in readmes"
   find . \
    -iname "readme.md" \
    -not -path "**/.git/**" \
    -not -path "**/.cargo/**" \
    | xargs -I {} \
    sed -i 's/${core-previous-version}/${core-version}/g' {}

   echo "bumping versions from ${core-previous-version} to ${core-version} in CLI"
   find . \
    -type f \
    -not -path "**/.git/**" \
    -path "./cli/*" \
    | xargs -I {} \
    sed -i 's/${core-previous-version}/${core-version}/g' {}

   echo "bumping node conductor version from ${node-conductor-previous-version} to ${node-conductor-version}"
   sed -i 's/^\s*version\s*=\s*"${node-conductor-previous-version}"\s*$/version = "${node-conductor-version}"/g' ./nodejs_conductor/native/Cargo.toml
   sed -i 's/"version": "${node-conductor-previous-version}"/"version": "${node-conductor-version}"/g' ./nodejs_conductor/package.json
   sed -i 's/"@holochain\/holochain-nodejs": "${node-conductor-previous-version}"/"@holochain\/holochain-nodejs": "${node-conductor-version}"/g' ./cli/src/cli/js-tests-scaffold/package.json
  '';

  # a few things should already be done by this point so precheck them :)
  release-details =
  ''
Release ${core-version}

current release process: ${release-process-url}

## Preparation

This should all be handled by `nix-shell --run hc-prepare-release`

- [x] develop is green
- [x] dev pulse commit for release candidate
- [x] core/hdk version updated in CLI scaffold
- [x] reviewed and updated the version numbers in Cargo.toml
- [x] holochain nodejs minor version bumped in CLI scaffold `package.json`

## Release docs

Run the basic linter with `nix-shell --run hc-lint-release-docs`

The linter will do some things for you and provide helpful debug info.

- [ ] reviewed and updated CHANGELOG
    - [ ] correct version + date
    - [ ] inserted template for next release
    - [ ] all root items have a PR link
- [ ] reviewed and updated README files
    - [ ] correct rust nightly version

Generate the github release notes with `nix-shell --run hc-generate-release-notes`

- [ ] review generated github release notes
    - [ ] correct medium post link for dev pulse
    - [ ] correct CHANGELOG link
    - [ ] hackmd link: {{URL}}
    - [ ] correct tags in blob links
    - [ ] correct rust nightly version
    - [ ] correct installation instructions
    - [ ] correct version number in binary file names

 ## Test builds

 Kick these off with `nix-shell --run hc-test-release`

 Every run of `hc-test-release` will cut new tags incrementally and trigger new builds on CI.

 Move on to "release docs" below while waiting for CI.

 - [ ] green core release test tag + linux/mac/windows artifacts on github
     - [ ] build: {{build URL}}
     - [ ] artifacts: {{artifacts URL}}
 - [ ] green node release test tag + linux/mac/windows artifacts on github
     - [ ] build: {{build URL}}
     - [ ] artifacts: {{artifacts URL}}

## Deploy artifacts

- [ ] release PR merged into `master`
- [ ] core release tag + linux/mac/windows artifacts on github
- [ ] node release tag + linux/mac/windows artifacts on github
- [ ] npm deploy
- [ ] release branch merged into `develop`
- [ ] test build artifacts deleted from github
- [ ] release notes copied into github
- [ ] `unknown` release assets renamed to `ubuntu`

## Finalise

- [ ] developer docs updated
- [ ] social medias
  '';
  hc-prepare-release-pr = pkgs.writeShellScriptBin "hc-prepare-release-pr"
  ''
  echo
  echo 'ensure github PR'
  git config --local hub.upstream ${repo}
  git config --local hub.forkrepo ${repo}
  git config --local hub.forkremote ${upstream}
  if [ "$(git rev-parse --abbrev-ref HEAD)" == "${release-branch}" ]
   then
    git add . && git commit -am 'Release ${core-version}'
    git push && git hub pull new -b 'master' -m '${release-details}' --no-triangular ${release-branch}
   else
    echo "current branch is not ${release-branch}!"
    exit 1
  fi
  '';

  hc-prepare-release = pkgs.writeShellScriptBin "hc-prepare-release"
  ''
   echo
   echo "IMPORTANT: make sure git-hub is setup on your machine"
   echo "1. Visit https://github.com/settings/tokens/new"
   echo "2. Generate a token called 'git-hub' with 'user' and 'repo' scopes"
   echo "3. git config --global hub.oauthtoken <token>"
   echo "4. git config --global hub.username <username>"
   echo
   echo "Current nix-shell config:"
   echo
   echo "pulse-url-hash: ${pulse-url-hash}"
   echo "pulse-version: ${pulse-version}"
   echo "pulse-commit: ${pulse-commit}"
   echo "core-previous-version: ${core-previous-version}"
   echo "core-version: ${core-version}"
   echo "node-conductor-previous-version: ${node-conductor-previous-version}"
   echo "node-conductor-version: ${node-conductor-version}"
   git hub --version
   echo
   read -r -p "Are you sure you want to cut a new release based on the current config in shell.nix? [y/N] " response
   case "$response" in
    [yY][eE][sS]|[yY])
     hc-prepare-pulse-tag \
     && hc-prepare-release-branch \
     && hc-prepare-crate-versions \
     && hc-prepare-release-pr \
     ;;
    *)
     exit 1
     ;;
   esac
  '';

  hc-test-release = pkgs.writeShellScriptBin "hc-test-release"
  ''
  echo
  echo "kicking off new test release build"
  echo

  git push || exit 1

  i="0"
  while [ $(git tag -l "test-$i-v${core-version}") ]
  do
   i=$[$i+1]
  done
  echo "tagging test-$i-v${core-version}"
  git tag -a "test-$i-v${core-version}" -m "Version ${core-version} release test $i"
  git push ${upstream} "test-$i-v${core-version}"

  n="0"
  while [ $(git tag -l "holochain-nodejs-test-$n-v${node-conductor-version}") ]
  do
   n=$[$n+1]
  done
  echo "tagging holochain-nodejs-test-$n-v${node-conductor-version}"
  git tag -a "holochain-nodejs-test-$n-v${node-conductor-version}" -m "Node conductor version ${node-conductor-version} release test $n"
  git push ${upstream} "holochain-nodejs-test-$n-v${node-conductor-version}"

  echo "testing tags pushed"
  echo "travis builds: https://travis-ci.com/holochain/holochain-rust/branches"
  echo "core artifacts: https://github.com/holochain/holochain-rust/releases/tag/test-$i-v${core-version}"
  echo "nodejs artifacts: https://github.com/holochain/holochain-rust/releases/tag/holochain-nodejs-test-$n-v${node-conductor-version}"
  '';

  changelog-template =
  ''\
\[Unreleased\]\n\n\
\#\#\# Added\n\n\
\#\#\# Changed\n\n\
\#\#\# Deprecated\n\n\
\#\#\# Removed\n\n\
\#\#\# Fixed\n\n\
\#\#\# Security\n\n\
  '';
  hc-lint-release-docs = pkgs.writeShellScriptBin "hc-lint-release-docs"
  ''
  echo
  echo "locking off changelog version"
  echo

  if ! $(grep -q "\[${core-version}\]" ./CHANGELOG.md)
   then
    echo "timestamping and retemplating changelog"
    sed -i "s/\[Unreleased\]/${changelog-template}\#\# \[${core-version}\] - $(date --iso --u)/" ./CHANGELOG.md
  fi

  echo
  echo "the following LOC in the CHANGELOG.md are missing a PR reference:"
  echo
  cat CHANGELOG.md | grep -E '^-\s' | grep -Ev '[0-9]\]'

  echo
  echo "the following LOC in README files reference the WRONG rust nightly date (should be ${date}):"
  echo
  find . \
   -iname "readme.*" \
   | xargs cat \
   | grep -E 'nightly-' \
   | grep -v '${date}'
  '';


  pulse-url = "https://medium.com/@holochain/${pulse-url-hash}";
  release-notes-template = ''
# ${core-version} release {{ release-date }}

{{ pulse-notes }}

See the [Dev Pulse](${pulse-url}) & [change log](https://github.com/holochain/holochain-rust/blob/release-${core-version}/CHANGELOG.md) for complete details.

## **Installation**

This release consists of binary builds of:

- the [`hc` development command-line tool](https://github.com/holochain/holochain-rust/blob/v${core-version}/cli/README.md)
- [`holochain` deployment conductor](https://github.com/holochain/holochain-rust/blob/v${core-version}/conductor/README.md) for different platforms.

To install, simply download and extract the binary for your platform.
See our [installation quick-start instructions](https://developer.holochain.org/start.html) for details.

Rust and NodeJS are both required for `hc` to build and test DNA:

- [Rust](https://www.rust-lang.org/en-US/install.html)
  - Must be `nightly-${date}` build with the WASM build target.
    Once you have first installed rustup:
    ```
    rustup toolchain install nightly-${date}
    rustup default nightly-${date}
    rustup target add wasm32-unknown-unknown --toolchain nightly-${date}
    ```
- [Node.js](https://nodejs.org) version 8 or higher
  - E2E tests for Holochain apps are written in Javascript client-side and executed in NodeJS through websockets
  - For further info, check out [the holochain-nodejs module](https://www.npmjs.com/package/@holochain/holochain-nodejs)

### **Which Binary?**

Download only the binaries for your operating system.

- MacOS: `cli-v${core-version}-x86_64-apple-darwin.tar.gz`
- Linux: `cli-v${core-version}-x86_64-ubuntu-linux-gnu.tar.gz`
- Windows:
  - mingw build system: `cli-v${core-version}-x86_64-pc-windows-gnu.tar.gz`
  - Visual Studio build system: `cli-v${core-version}-x86_64-pc-windows-msvc.tar.gz`

All binaries are for 64-bit operating systems.
32-bit systems are NOT supported.
  '';
  hc-generate-release-notes = pkgs.writeShellScriptBin "hc-generate-release-notes"
  ''
   TEMPLATE=$( echo '${release-notes-template}' )

   DATE_PLACEHOLDER='{{ release-date }}'
   DATE=$( date --iso -u )
   WITH_DATE=''${TEMPLATE/$DATE_PLACEHOLDER/$DATE}

   PULSE_PLACEHOLDER='{{ pulse-notes }}'
   # magic
   # gets a markdown version of pulse
   # greps for everything from summary to details (not including details heading)
   # deletes null characters that throw warnings in bash
   PULSE_NOTES=$( curl -s https://md.unmediumed.com/${pulse-url} | grep -Pzo "(?s)(###.*Summary.*)(?=###.*Details)" | tr -d '\0' )
   WITH_NOTES=''${WITH_DATE/$PULSE_PLACEHOLDER/$PULSE_NOTES}
   echo "$WITH_NOTES"
  '';

  hc-check-release-artifacts = pkgs.writeShellScriptBin "hc-check-release-artifacts"
  ''
  echo
  echo "Checking core artifacts"
  echo

  i="0"
  while [ $(git tag -l "test-$i-v${core-version}") ]
  do
   i=$[$i+1]
  done
  i=$[$i-1]

  core_binaries=( "cli" "conductor" )
  core_platforms=( "apple-darwin" "pc-windows-gnu" "pc-windows-msvc" "unknown-linux-gnu" )
  deployments=( "test-$i-" "" )

  for deployment in "''${deployments[@]}"
  do
   for binary in "''${core_binaries[@]}"
   do
    for platform in "''${core_platforms[@]}"
    do
     file="$binary-''${deployment}v${core-version}-x86_64-$platform.tar.gz"
     release="''${deployment}v${core-version}"
     url="https://github.com/holochain/holochain-rust/releases/download/$release/$file"
     echo
     echo "pinging $file for release $release..."
     if curl -Is "$url" | grep -q "HTTP/1.1 302 Found"
      then echo "FOUND ✔"
      else echo "NOT FOUND ⨯"
     fi
     echo
    done
   done
  done

  echo
  echo "Checking node conductor artifacts"
  echo

  n="0"
  while [ $(git tag -l "holochain-nodejs-test-$n-v${node-conductor-version}") ]
  do
   n=$[$n+1]
  done
  n=$[$n-1]

  node_versions=( "57" "64" "67" )
  conductor_platforms=( "darwin" "linux" "win32" )
  deployments=( "test-$n-" "" )

  for deployment in "''${deployments[@]}"
  do
   for node_version in "''${node_versions[@]}"
   do
    for platform in "''${conductor_platforms[@]}"
    do
     file="index-v${node-conductor-version}-node-v''${node_version}-''${platform}-x64.tar.gz"
     release="holochain-nodejs-''${deployment}v${node-conductor-version}"
     url="https://github.com/holochain/holochain-rust/releases/download/$release/$file"
     echo
     echo "pinging $file for release $release..."
     if curl -Is "$url" | grep -q "HTTP/1.1 302 Found"
      then echo "FOUND ✔"
      else echo "NOT FOUND ⨯"
     fi
     echo
    done
   done
  done
  '';

  hc-do-release = pkgs.writeShellScriptBin "hc-do-release"
  ''
   git checkout master
   git pull
   git tag -a v${core-version} -m 'Version ${core-version}'
   git tag -a holochain-nodejs-v${node-conductor-version} -m 'holochain-nodejs-v${node-conductor-version}'
   git push ${upstream} v${core-version}
   git push ${upstream} holochain-nodejs-v${node-conductor-version}
  '';

  hc-npm-deploy = pkgs.writeShellScriptBin "hc-npm-deploy"
  ''
   git checkout holochain-nodejs-v${node-conductor-version}
   npm login
   cd nodejs_conductor
   yarn install --ignore-scripts
   RUST_SODIUM_DISABLE_PIE=1 node ./publish.js --publish
  '';
  hc-npm-check-version = pkgs.writeShellScriptBin "hc-npm-check-version"
  ''
  echo
  echo "Checking deployed nodejs_conductor version."
  deployed=$( npm v @holochain/holochain-nodejs dist-tags.latest )
  if [ $deployed == ${node-conductor-version} ]
   then echo "Version ${node-conductor-version} deployed ✔"
   else echo "Not deployed. $deployed found instead. ⨯"
  fi
  echo
  '';

  hc-release-cleanup = pkgs.writeShellScriptBin "hc-release-cleanup"
  ''
   export GITHUB_USER='holochain'
   export GITHUB_REPO='holochain-rust'
   export GITHUB_TOKEN=$( git config --get hub.oauthtoken )

   echo
   echo 'Injecting medium summary/highlights into github release notes'
   echo
   github-release -v edit --tag v${core-version} --name v${core-version} --description "$( hc-generate-release-notes )" --pre-release

   echo
   echo 'Deleting releases on test-* tags'
   echo
   github-release info \
    | grep -Po "(?<=name: ')(test-\S*)(?=',)" \
    | xargs -I {} github-release -v delete --tag {}

   echo
   echo 'Deleting releases on holochain-nodejs-test-* tags'
   echo
   github-release info \
    | grep -Po "(?<=name: ')(holochain-nodejs-test-\S*)(?=',)" \
    | xargs -I {} github-release -v delete --tag {}

   echo
   echo 'ensure github PR against develop'
   echo
   git config --local hub.upstream ${repo}
   git config --local hub.forkrepo ${repo}
   git config --local hub.forkremote ${upstream}
   if [ "$(git rev-parse --abbrev-ref HEAD)" == "${release-branch}" ]
    then
     git add . && git commit -am 'Release ${core-version}'
     git push && git hub pull new -b 'develop' -m 'Merge release ${core-version} back to develop' --no-triangular ${release-branch}
    else
     echo "current branch is not ${release-branch}!"
     exit 1
   fi
  '';

in
with pkgs;
stdenv.mkDerivation rec {
  name = "holochain-rust-environment";

  buildInputs = [

    # https://github.com/NixOS/pkgs/blob/master/doc/languages-frameworks/rust.section.md
    binutils gcc gnumake openssl pkgconfig coreutils

    cmake
    python
    pkgconfig
    rust-build

    nodejs-8_x
    yarn

    hc-node-flush
    hc-cargo-flush
    hc-cargo-lock-flush
    hc-flush-all

    hc-cargo-toml-set-ver
    hc-cargo-toml-test-ver
    hc-cargo-toml-grep-unpinned

    hc-build-wasm
    hc-test

    hc-tarpaulin

    hc-install-tarpaulin
    hc-install-fmt
    hc-install-edit
    hc-install-cli
    hc-install-conductor
    hc-install-node-conductor

    hc-test-cli
    hc-test-app-spec
    hc-test-node-conductor

    hc-fmt
    hc-fmt-check

    hc-test-all

    # dev tooling
    git

    # curl needed to push to codecov
    curl
    circleci-cli
    hc-codecov
    ci

    # release tooling
    gitAndTools.git-hub
    github-release
    hc-prepare-pulse-tag
    hc-prepare-release-branch
    hc-prepare-release-pr
    hc-prepare-crate-versions
    hc-check-release-artifacts

    hc-prepare-release
    hc-test-release
    hc-lint-release-docs
    hc-generate-release-notes

    hc-do-release

    hc-npm-deploy
    hc-npm-check-version

    hc-release-cleanup

  ] ++ lib.optionals stdenv.isDarwin [ frameworks.Security frameworks.CoreFoundation frameworks.CoreServices ];

  # https://github.com/rust-unofficial/patterns/blob/master/anti_patterns/deny-warnings.md
  # https://llogiq.github.io/2017/06/01/perf-pitfalls.html
  RUSTFLAGS = "-D warnings -Z external-macro-backtrace -Z thinlto -C codegen-units=16 -C opt-level=z";
  CARGO_INCREMENTAL = "1";
  # https://github.com/rust-lang/cargo/issues/4961#issuecomment-359189913
  # RUST_LOG = "info";

  # non-nixos OS can have a "dirty" setup with rustup installed for the current
  # user.
  # `nix-shell` can inherit this e.g. through sourcing `.bashrc`.
  # even `nix-shell --pure` will still source some files and inherit paths.
  # for those users we can at least give the OS a clue that we want our pinned
  # rust version through this environment variable.
  # https://github.com/rust-lang/rustup.rs#environment-variables
  # https://github.com/NixOS/nix/issues/903
  RUSTUP_TOOLCHAIN = "nightly-${date}";

  DARWIN_NIX_LDFLAGS = if stdenv.isDarwin then "-F${frameworks.CoreFoundation}/Library/Frameworks -framework CoreFoundation " else "";

  shellHook = ''
   # cargo installs things to the user's home so we need it on the path
   export PATH=$PATH:~/.cargo/bin
   export HC_TARGET_PREFIX=~/nix-holochain/
   export NIX_LDFLAGS="$DARWIN_NIX_LDFLAGS$NIX_LDFLAGS"
  '';
}
