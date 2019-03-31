let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  pkgs = import <nixpkgs> {
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

  repo = "holochain/holochain-rust";
  upstream = "origin";
  pulse-version = "22";
  pulse-commit = "0a524d3be580249d54cf5073591fa9fe1f30a174";
  core-previous-version = "0.0.8-alpha";
  core-version = "0.0.9-alpha";
  node-conductor-previous-version = "0.0.8-alpha";
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
   echo
   echo "bumping core version from ${core-previous-version} to ${core-version}"
   echo
   find . \
   -name "Cargo.toml" \
   -not -path "**/.cargo/**" \
   -not -path "./nodejs_conductor*" \
   | xargs -I {} \
   sed -i 's/^\s*version\s*=\s*"${core-previous-version}"\s*$/version = "${core-version}"/g' {}

   echo
   echo "bumping node conductor version from ${node-conductor-previous-version} to ${node-conductor-version}"
   echo
   find . \
   -name "Cargo.toml" \
   -path "./nodejs_conductor*"
  '';

  # a few things should already be done by this point so precheck them :)
  release-details =
  ''
Release ${core-version}

- [x] develop is green
- [x] dev pulse commit for release candidate
- [ ] core/hdk version updated in CLI scaffold
- [ ] reviewed and updated the version numbers in Cargo.toml
- [ ] holochain nodejs minor version bumped in CLI scaffold `package.json`
- [ ] reviewed and updated CHANGELOG
- [ ] reviewed and updated README files
- [ ] written github release notes
    - [ ] correct medium post link for dev pulse
    - [ ] correct CHANGELOG link
    - [ ] hackmd link: {{URL}}
    - [ ] correct tags in blob links
    - [ ] correct rust nightly version
    - [ ] correct installation instructions
    - [ ] correct version number in binary file names
- [ ] green core release test tag + linux/mac/windows artifacts on github
    - [ ] build: {{build URL}}
    - [ ] artifacts: {{artifacts URL}}
- [ ] green node release test tag + linux/mac/windows artifacts on github
    - [ ] build: {{build URL}}
    - [ ] artifacts: {{artifacts URL}}
- [ ] QA: artifacts install on supported platforms
- [ ] QA: @Connoropolous :+1: docs
- [ ] QA: hApps run
- [ ] QA: hc generate run
- [ ] release PR merged into `master`
- [ ] core release tag + linux/mac/windows artifacts on github
- [ ] node release tag + linux/mac/windows artifacts on github
- [ ] npm deploy
- [ ] release branch merged into `develop`
- [ ] test build artifacts deleted from github
- [ ] release notes copied into github
- [ ] `unknown` release assets renamed to `ubuntu`
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
    git hub pull new -b 'master' -m '${release-details}' --no-triangular ${release-branch}
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
   echo "pulse-version: ${pulse-version}"
   echo "pulse-commit: ${pulse-commit}"
   echo "core-previous-version: ${core-previous-version}"
   echo "core-version: ${core-version}"
   echo "node-conductor-previous-version: ${node-conductor-previous-version}"
   echo "node-conductor-version: ${node-conductor-version}"
   echo
   read -r -p "Are you sure you want to cut a new release based on the current config in shell.nix? [y/N] " response
   case "$response" in
    [yY][eE][sS]|[yY])
     git hub --version \
     && hc-prepare-pulse-tag \
     && hc-prepare-release-branch \
     && hc-prepare-release-pr \
     ;;
    *)
     exit 1
     ;;
   esac
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
    hc-prepare-pulse-tag
    hc-prepare-release-branch
    hc-prepare-release-pr
    hc-prepare-release
    hc-prepare-crate-versions

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
