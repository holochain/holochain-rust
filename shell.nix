let

  pkgs = import ./holonix/nixpkgs/nixpkgs.nix;
  rust = import ./holonix/rust/config.nix;
  release = import ./holonix/release/config.nix;
  git = import ./holonix/git/config.nix;

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

  # runs all standard tests and reports code coverage
  hc-codecov = pkgs.writeShellScriptBin "hc-codecov"
  ''
   hc-install-tarpaulin && \
   hc-tarpaulin && \
   bash <(curl -s https://codecov.io/bash);
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
   && hc-conductor-node-install \
   && hc-test-app-spec
  '';

  hc-prepare-crate-versions = pkgs.writeShellScriptBin "hc-prepare-crate-versions"
  ''
   echo "bumping core version from ${release.core.version.previous} to ${release.core.version.current} in Cargo.toml"
   find . \
    -name "Cargo.toml" \
    -not -path "**/.git/**" \
    -not -path "**/.cargo/**" \
    -not -path "./nodejs_conductor*" \
    | xargs -I {} \
    sed -i 's/^\s*version\s*=\s*"${release.core.version.previous}"\s*$/version = "${release.core.version.current}"/g' {}

   echo "bumping core versions from ${release.core.version.previous} to ${release.core.version.current} in readmes"
   find . \
    -iname "readme.md" \
    -not -path "**/.git/**" \
    -not -path "**/.cargo/**" \
    | xargs -I {} \
    sed -i 's/${release.core.version.previous}/${release.core.version.current}/g' {}

   echo "bumping versions from ${release.core.version.previous} to ${release.core.version.current} in CLI"
   find . \
    -type f \
    -not -path "**/.git/**" \
    -path "./cli/*" \
    | xargs -I {} \
    sed -i 's/${release.core.version.previous}/${release.core.version.current}/g' {}

   echo "bumping node conductor version from ${release.node-conductor.version.previous} to ${release.node-conductor.version.current}"
   sed -i 's/^\s*version\s*=\s*"${release.node-conductor.version.previous}"\s*$/version = "${release.node-conductor.version.current}"/g' ./nodejs_conductor/native/Cargo.toml
   sed -i 's/"version": "${release.node-conductor.version.previous}"/"version": "${release.node-conductor.version.current}"/g' ./nodejs_conductor/package.json
   sed -i 's/"@holochain\/holochain-nodejs": "${release.node-conductor.version.previous}"/"@holochain\/holochain-nodejs": "${release.node-conductor.version.current}"/g' ./cli/src/cli/js-tests-scaffold/package.json
  '';

  # a few things should already be done by this point so precheck them :)
  release-details =
  ''
Release ${release.core.version.current}

current release process: ${release.process-url}

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
  - artifacts: https://github.com/holochain/holochain-rust/releases/tag/${release.core.tag}
- [ ] node release tag + linux/mac/windows artifacts on github
  - travis build: {{ build url }}
  - artifacts: https://github.com/holochain/holochain-rust/releases/tag/${release.node-conductor.tag}
- [ ] all release artifacts found by `hc-check-release-artifacts`
- [ ] npmjs deploy with `hc-release-npm-deploy` then `hc-release-npm-check-version`
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
  git config --local hub.upstream ${git.github.repo}
  git config --local hub.forkrepo ${git.github.repo}
  git config --local hub.forkremote ${git.github.upstream}
  if [ "$(git rev-parse --abbrev-ref HEAD)" == "${release.branch}" ]
   then
    git add . && git commit -am 'Release ${release.core.version.current}'
    git push && git hub pull new -b 'master' -m '${release-details}' --no-triangular ${release.branch}
   else
    echo "current branch is not ${release.branch}!"
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
   echo "pulse-url-hash: ${release.pulse.url-hash}"
   echo "pulse-version: ${release.pulse.version}"
   echo "pulse-commit: ${release.pulse.commit}"
   echo "core-previous-version: ${release.core.version.previous}"
   echo "core-version: ${release.core.version.current}"
   echo "node-conductor-previous-version: ${release.node-conductor.version.previous}"
   echo "node-conductor-version: ${release.node-conductor.version.current}"
   git hub --version
   echo
   read -r -p "Are you sure you want to cut a new release based on the current config in shell.nix? [y/N] " response
   case "$response" in
    [yY][eE][sS]|[yY])
     hc-release-pulse-tag \
     && hc-release-git-branch \
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
  echo "releasing core ${release.core.tag}"
  echo

  echo "tagging ${release.core.tag}"
  git tag -a ${release.core.tag} -m "Version ${release.core.tag}"
  git push ${git.github.upstream} ${release.core.tag}

  echo
  echo "releasing node conductor ${release.node-conductor.tag}"
  echo

  echo "tagging ${release.node-conductor.tag}"
  git tag -a ${release.node-conductor.tag} -m "Node conductor version ${release.node-conductor.tag}"
  git push ${git.github.upstream} ${release.node-conductor.tag}

  echo "release tags pushed"
  echo "travis builds: https://travis-ci.com/holochain/holochain-rust/branches"
  echo "core artifacts: https://github.com/holochain/holochain-rust/releases/tag/${release.core.tag}"
  echo "nodejs artifacts: https://github.com/holochain/holochain-rust/releases/tag/${release.node-conductor.tag}"
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

  if ! $(grep -q "\[${release.core.version.current}\]" ./CHANGELOG.md)
   then
    echo "timestamping and retemplating changelog"
    sed -i "s/\[Unreleased\]/${changelog-template}\#\# \[${release.core.version.current}\] - $(date --iso --u)/" ./CHANGELOG.md
  fi
  '';

  hc-readme-grep-nightly = pkgs.writeShellScriptBin "hc-readme-grep-nightly"
  ''
  find . \
   -iname "readme.*" \
   | xargs cat \
   | grep -E 'nightly-' \
   | grep -v '${rust.nightly-date}' \
   | cat
  '';

  release-notes-template = ''
# ${release.core.version.current} release {{ release-date }}

{{ pulse-notes }}

See the [Dev Pulse](${release.pulse.url}) & [change log](https://github.com/holochain/holochain-rust/blob/release-${release.core.version.current}/CHANGELOG.md) for complete details.

## **Installation**

This release consists of binary builds of:

- the [`hc` development command-line tool](https://github.com/holochain/holochain-rust/blob/${release.core.tag}/cli/README.md)
- [`holochain` deployment conductor](https://github.com/holochain/holochain-rust/blob/${release.core.tag}/conductor/README.md) for different platforms.

To install, simply download and extract the binary for your platform.
See our [installation quick-start instructions](https://developer.holochain.org/start.html) for details.

Rust and NodeJS are both required for `hc` to build and test DNA:

- [Rust](https://www.rust-lang.org/en-US/install.html)
  - Must be `nightly-${rust.nightly-date}` build with the WASM build target.
    Once you have first installed rustup:
    ```
    rustup toolchain install nightly-${rust.nightly-date}
    rustup default nightly-${rust.nightly-date}
    rustup target add wasm32-unknown-unknown --toolchain nightly-${rust.nightly-date}
    ```
- [Node.js](https://nodejs.org) version 8 or higher
  - E2E tests for Holochain apps are written in Javascript client-side and executed in NodeJS through websockets
  - For further info, check out [the holochain-nodejs module](https://www.npmjs.com/package/@holochain/holochain-nodejs)

### **Which Binary?**

Download only the binaries for your operating system.

- MacOS: `cli-${release.core.tag}-x86_64-apple-darwin.tar.gz`
- Linux: `cli-${release.core.tag}-x86_64-ubuntu-linux-gnu.tar.gz`
- Windows:
  - mingw build system: `cli-${release.core.tag}-x86_64-pc-windows-gnu.tar.gz`
  - Visual Studio build system: `cli-${release.core.tag}-x86_64-pc-windows-msvc.tar.gz`

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
   PULSE_NOTES=$( curl -s https://md.unmediumed.com/${release.pulse.url} | grep -Pzo "(?s)(###\s+\**Summary.*)(?=###\s+\**Details)" | tr -d '\0' )
   WITH_NOTES=''${WITH_DATE/$PULSE_PLACEHOLDER/$PULSE_NOTES}
   echo "$WITH_NOTES"
  '';

  hc-check-release-artifacts = pkgs.writeShellScriptBin "hc-check-release-artifacts"
  ''
  echo
  echo "Checking core artifacts"
  echo

  echo
  echo "checking ${release.core.tag}"
  echo

  core_binaries=( "cli" "conductor" )
  core_platforms=( "apple-darwin" "pc-windows-gnu" "pc-windows-msvc" "unknown-linux-gnu" )

  for binary in "''${core_binaries[@]}"
  do
   for platform in "''${core_platforms[@]}"
   do
    file="$binary-${release.core.tag}-x86_64-$platform.tar.gz"
    url="https://github.com/holochain/holochain-rust/releases/download/${release.core.tag}/$file"
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
  echo "checking ${release.node-conductor.tag}"
  echo

  node_versions=( "57" "64" "67" )
  conductor_platforms=( "darwin" "linux" "win32" )

  for node_version in "''${node_versions[@]}"
  do
   for platform in "''${conductor_platforms[@]}"
   do
    file="index-v${release.node-conductor.version.current}-node-v''${node_version}-''${platform}-x64.tar.gz"
    url="https://github.com/holochain/holochain-rust/releases/download/${release.node-conductor.tag}/$file"
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

  hc-release-merge-back = pkgs.writeShellScriptBin "hc-release-merge-back"
  ''
   echo
   echo 'ensure github PR against develop'
   echo
   git config --local hub.upstream ${git.github.repo}
   git config --local hub.forkrepo ${git.github.repo}
   git config --local hub.forkremote ${git.github.upstream}
   if [ "$(git rev-parse --abbrev-ref HEAD)" == "${release.branch}" ]
    then
     git add . && git commit -am 'Release ${release.core.version.current}'
     git push && git hub pull new -b 'develop' -m 'Merge release ${release.core.version.current} back to develop' --no-triangular ${release.branch}
    else
     echo "current branch is not ${release.branch}!"
     exit 1
   fi

   export GITHUB_USER='holochain'
   export GITHUB_REPO='holochain-rust'
   export GITHUB_TOKEN=$( git config --get hub.oauthtoken )

   echo
   echo 'Setting release to pre-release state'
   echo
   github-release -v edit --tag ${release.core.tag} --pre-release
  '';

  hc-release-pulse-sync = pkgs.writeShellScriptBin "hc-release-pulse-sync"
  ''
   export GITHUB_USER='holochain'
   export GITHUB_REPO='holochain-rust'
   export GITHUB_TOKEN=$( git config --get hub.oauthtoken )

   echo
   echo 'Injecting medium summary/highlights into github release notes'
   echo
   github-release -v edit --tag ${release.core.tag} --name ${release.core.tag} --description "$( hc-generate-release-notes )" --pre-release
  '';

  build-release-artifact = params:
  ''
   export artifact_name=`sed "s/unknown/generic/g" <<< "${params.path}-${release.core.version.current}-${rust.generic-linux-target}"`
   echo
   echo "building $artifact_name..."
   echo

   CARGO_INCREMENTAL=0 cargo rustc --manifest-path ${params.path}/Cargo.toml --target ${rust.generic-linux-target} --release -- -C lto
   mkdir -p dist/$artifact_name
   cp target/${rust.generic-linux-target}/release/${params.name} ${params.path}/LICENSE ${params.path}/README.md dist/$artifact_name
   tar -C dist/$artifact_name -czf dist/$artifact_name.tar.gz . && rm -rf dist/$artifact_name
  '';
  build-release-paramss = [
                           {
                            path = "cli";
                            name = "hc";
                           }
                           {
                            path = "conductor";
                            name = "holochain";
                           }
                          ];
  build-node-conductor-artifact = node-version:
  ''
   hc-node-flush
   echo
   echo "building conductor for node ${node-version}..."
   echo

   node -v
   ./scripts/build_nodejs_conductor.sh
   cp nodejs_conductor/bin-package/index-v${release.node-conductor.version.current}-node-v57-linux-x64.tar.gz dist
  '';
  build-node-conductor-versions = [ "nodejs-8_x" ];
  hc-build-release-artifacts = pkgs.writeShellScriptBin "hc-build-release-artifacts"
  ''
   ${pkgs.lib.concatMapStrings (params: build-release-artifact params) build-release-paramss}
   ${pkgs.lib.concatMapStrings (node-version: build-node-conductor-artifact node-version) build-node-conductor-versions}
  '';

in
with pkgs;
stdenv.mkDerivation rec {
  name = "holochain-rust-environment";

  buildInputs = [

    coreutils which

    cmake
    python

    qt5.qmake

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

    hc-test-cli
    hc-test-app-spec
    hc-test-node-conductor
    hc-test-c-bindings

    hc-test-all

    # curl needed to push to codecov
    curl
    hc-codecov

    hc-prepare-release-pr
    hc-prepare-crate-versions
    hc-check-release-artifacts

    hc-prepare-release
    hc-changelog-grep-pr-references
    hc-ensure-changelog-version
    hc-generate-release-notes
    hc-readme-grep-nightly
    hc-build-release-artifacts

    hc-do-release

    hc-release-merge-back
    hc-release-pulse-sync

  ]

  # root build inputs
  ++ import ./holonix/build.nix;

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
  RUSTUP_TOOLCHAIN = "nightly-${rust.nightly-date}";

  DARWIN_NIX_LDFLAGS = if stdenv.isDarwin then "-F${frameworks.CoreFoundation}/Library/Frameworks -framework CoreFoundation " else "";

  OPENSSL_STATIC = "1";

  shellHook = ''
   # cargo installs things to the user's home so we need it on the path
   export PATH=$PATH:~/.cargo/bin
   export HC_TARGET_PREFIX=~/nix-holochain/
   export NIX_LDFLAGS="$DARWIN_NIX_LDFLAGS$NIX_LDFLAGS"
  '';
}
