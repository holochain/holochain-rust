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
  release-process-url = "https://hackmd.io/pt72afqYTWat7cuNqpAFjw";
  repo = "holochain/holochain-rust";
  upstream = "origin";

  # the unique hash at the end of the medium post url
  # e.g. https://medium.com/@holochain/foos-and-bars-4867d777de94
  # would be 4867d777de94
  pulse-url-hash = "d387ffcfac72";
  pulse-version = "24";
  pulse-commit = "494c21b9dc7927b7b171533cc20c4d39bd92b45c";

  core-previous-version = "0.0.10-alpha2";
  core-version = "0.0.11-alpha1";

  node-conductor-previous-version = "0.4.9-alpha2";
  node-conductor-version = "0.4.10-alpha1";

  core-tag = "v${core-version}";
  node-conductor-tag = "holochain-nodejs-v${node-conductor-version}";

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
   ./scripts/build_nodejs_conductor.sh;
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

  hc-changelog-grep-pr-references = pkgs.writeShellScriptBin "hc-changelog-grep-pr-references"
  ''
  cat CHANGELOG.md | grep -E '^-\s' | grep -Ev '[0-9]\]' | cat
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
  # simplified version of the c bindings test command in makefile
  # hardcodes hc_dna to test rather than looping/scanning like make does
  # might want to make this more sophisticated if we end up with many tests
  hc-test-c-bindings = pkgs.writeShellScriptBin "hc-test-c-bindings"
  ''
  cargo build -p holochain_dna_c_binding
  ( cd c_binding_tests/hc_dna && qmake -o $@Makefile $@qmake.pro && make )
  ./target/debug/c_binding_tests/hc_dna/test_executable
  '';
  hc-test = pkgs.writeShellScriptBin "hc-test"
  ''
   hc-build-wasm
   HC_SIMPLE_LOGGER_MUTE=1 RUST_BACKTRACE=1 cargo test --all --release --target-dir "$HC_TARGET_PREFIX"target "$1";
   hc-test-c-bindings
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

First checkout `develop` and `git pull` to ensure you are up to date locally.
Then run `nix-shell --run hc-prepare-release`

- [x] develop is green
- [x] correct dev pulse commit + version + url hash
- [x] correct core version
- [x] correct node conductor
- [x] correct release process url

## PR into master

- [ ] reviewed and updated CHANGELOG
- [ ] release PR merged into `master`

## Build and deploy release artifacts

- [ ] release cut from `master` with `hc-do-release`
- [ ] core release tag + linux/mac/windows artifacts on github
  - travis build: {{ build url }}
  - artifacts: https://github.com/holochain/holochain-rust/releases/tag/${core-tag}
- [ ] node release tag + linux/mac/windows artifacts on github
  - travis build: {{ build url }}
  - artifacts: https://github.com/holochain/holochain-rust/releases/tag/${node-conductor-tag}
- [ ] all release artifacts found by `hc-check-release-artifacts`
- [ ] npmjs deploy with `hc-npm-deploy` then `hc-npm-check-version`
- [ ] `unknown` release assets renamed to `ubuntu`

## PR into develop

- [ ] `hc-release-merge-back`
- [ ] `develop` PR changelog cleaned up
  - [ ] no new items from `develop` under recently released changelog header
- [ ] merge `develop` PR

## Finalise

- [ ] dev pulse is live on medium
- [ ] `hc-release-pulse-sync`

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
     && hc-ensure-changelog-version \
     && hc-prepare-release-pr \
     ;;
    *)
     exit 1
     ;;
   esac
  '';

  hc-do-release = pkgs.writeShellScriptBin "hc-do-release"
  ''
  echo
  echo "kicking off release"
  echo

  git checkout master
  git pull

  echo
  echo "releasing core ${core-tag}"
  echo

  echo "tagging ${core-tag}"
  git tag -a ${core-tag} -m "Version ${core-tag}"
  git push ${upstream} ${core-tag}

  echo
  echo "releasing node conductor ${node-conductor-tag}"
  echo

  echo "tagging ${node-conductor-tag}"
  git tag -a ${node-conductor-tag} -m "Node conductor version ${node-conductor-tag}"
  git push ${upstream} ${node-conductor-tag}

  echo "release tags pushed"
  echo "travis builds: https://travis-ci.com/holochain/holochain-rust/branches"
  echo "core artifacts: https://github.com/holochain/holochain-rust/releases/tag/${core-tag}"
  echo "nodejs artifacts: https://github.com/holochain/holochain-rust/releases/tag/${node-conductor-tag}"
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
  hc-ensure-changelog-version = pkgs.writeShellScriptBin "hc-ensure-changelog-version"
  ''
  echo
  echo "locking off changelog version"
  echo

  if ! $(grep -q "\[${core-version}\]" ./CHANGELOG.md)
   then
    echo "timestamping and retemplating changelog"
    sed -i "s/\[Unreleased\]/${changelog-template}\#\# \[${core-version}\] - $(date --iso --u)/" ./CHANGELOG.md
  fi
  '';

  hc-readme-grep-nightly = pkgs.writeShellScriptBin "hc-readme-grep-nightly"
  ''
  find . \
   -iname "readme.*" \
   | xargs cat \
   | grep -E 'nightly-' \
   | grep -v '${date}' \
   | cat
  '';

  pulse-url = "https://medium.com/@holochain/${pulse-url-hash}";
  release-notes-template = ''
# ${core-version} release {{ release-date }}

{{ pulse-notes }}

See the [Dev Pulse](${pulse-url}) & [change log](https://github.com/holochain/holochain-rust/blob/release-${core-version}/CHANGELOG.md) for complete details.

## **Installation**

This release consists of binary builds of:

- the [`hc` development command-line tool](https://github.com/holochain/holochain-rust/blob/${core-tag}/cli/README.md)
- [`holochain` deployment conductor](https://github.com/holochain/holochain-rust/blob/${core-tag}/conductor/README.md) for different platforms.

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

- MacOS: `cli-${core-tag}-x86_64-apple-darwin.tar.gz`
- Linux: `cli-${core-tag}-x86_64-ubuntu-linux-gnu.tar.gz`
- Windows:
  - mingw build system: `cli-${core-tag}-x86_64-pc-windows-gnu.tar.gz`
  - Visual Studio build system: `cli-${core-tag}-x86_64-pc-windows-msvc.tar.gz`

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
   PULSE_NOTES=$( curl -s https://md.unmediumed.com/${pulse-url} | grep -Pzo "(?s)(###.*Summary.*)(?=###\s+\**Details)" | tr -d '\0' )
   WITH_NOTES=''${WITH_DATE/$PULSE_PLACEHOLDER/$PULSE_NOTES}
   echo "$WITH_NOTES"
  '';

  hc-check-release-artifacts = pkgs.writeShellScriptBin "hc-check-release-artifacts"
  ''
  echo
  echo "Checking core artifacts"
  echo

  echo
  echo "checking ${core-tag}"
  echo

  core_binaries=( "cli" "conductor" )
  core_platforms=( "apple-darwin" "pc-windows-gnu" "pc-windows-msvc" "unknown-linux-gnu" )

  for binary in "''${core_binaries[@]}"
  do
   for platform in "''${core_platforms[@]}"
   do
    file="$binary-${core-tag}-x86_64-$platform.tar.gz"
    url="https://github.com/holochain/holochain-rust/releases/download/${core-tag}/$file"
    echo
    echo "pinging $file for release $release..."
    if curl -Is "$url" | grep -q "HTTP/1.1 302 Found"
     then echo "FOUND ✔"
     else echo "NOT FOUND ⨯"
    fi
    echo
   done
  done

  echo
  echo "Checking node conductor artifacts"
  echo

  echo
  echo "checking ${node-conductor-tag}"
  echo

  node_versions=( "57" "64" "67" )
  conductor_platforms=( "darwin" "linux" "win32" )

  for node_version in "''${node_versions[@]}"
  do
   for platform in "''${conductor_platforms[@]}"
   do
    file="index-v${node-conductor-version}-node-v''${node_version}-''${platform}-x64.tar.gz"
    url="https://github.com/holochain/holochain-rust/releases/download/${node-conductor-tag}/$file"
    echo
    echo "pinging $file for release $release..."
    if curl -Is "$url" | grep -q "HTTP/1.1 302 Found"
     then echo "FOUND ✔"
     else echo "NOT FOUND ⨯"
    fi
    echo
   done
  done
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

  hc-release-merge-back = pkgs.writeShellScriptBin "hc-release-merge-back"
  ''
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

   export GITHUB_USER='holochain'
   export GITHUB_REPO='holochain-rust'
   export GITHUB_TOKEN=$( git config --get hub.oauthtoken )

   echo
   echo 'Setting release to pre-release state'
   echo
   github-release -v edit --tag ${core-tag} --pre-release
  '';

  hc-release-pulse-sync = pkgs.writeShellScriptBin "hc-release-pulse-sync"
  ''
   export GITHUB_USER='holochain'
   export GITHUB_REPO='holochain-rust'
   export GITHUB_TOKEN=$( git config --get hub.oauthtoken )

   echo
   echo 'Injecting medium summary/highlights into github release notes'
   echo
   github-release -v edit --tag ${core-tag} --name ${core-tag} --description "$( hc-generate-release-notes )" --pre-release
  '';

in
with pkgs;
stdenv.mkDerivation rec {
  name = "holochain-rust-environment";

  buildInputs = [
    # https://github.com/NixOS/pkgs/blob/master/doc/languages-frameworks/rust.section.md
    binutils gcc gnumake openssl pkgconfig coreutils which

    # for openssl static installation
    perl

    cmake
    python
    pkgconfig
    rust-build

    qt5.qmake

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
    hc-test-c-bindings

    hc-fmt
    hc-fmt-check

    hc-test-all

    # dev tooling
    git

    # curl needed to push to codecov
    curl
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
    hc-changelog-grep-pr-references
    hc-ensure-changelog-version
    hc-generate-release-notes
    hc-readme-grep-nightly

    hc-do-release

    hc-npm-deploy
    hc-npm-check-version

    hc-release-merge-back
    hc-release-pulse-sync

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

  OPENSSL_STATIC = "1";

  shellHook = ''
   # cargo installs things to the user's home so we need it on the path
   export PATH=$PATH:~/.cargo/bin
   export HC_TARGET_PREFIX=~/nix-holochain/
   export NIX_LDFLAGS="$DARWIN_NIX_LDFLAGS$NIX_LDFLAGS"
  '';
}
