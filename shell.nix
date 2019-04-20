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

  hc-test-all = pkgs.writeShellScriptBin "hc-test-all"
  ''
   hc-fmt-check \
   && hc-qt-c-bindings-test \
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

    # I forgot what these are for!
    # Reinstate and organise them ᕙ༼*◕_◕*༽ᕤ
    # coreutils
    # cmake
    # python

    hc-cargo-toml-set-ver
    hc-cargo-toml-test-ver

    hc-tarpaulin

    hc-install-tarpaulin
    hc-install-fmt
    hc-install-edit
    hc-install-cli
    hc-install-conductor

    hc-test-cli
    hc-test-app-spec
    hc-test-node-conductor

    hc-test-all

    # curl needed to push to codecov
    curl
    hc-codecov

    hc-prepare-crate-versions

    hc-changelog-grep-pr-references
    hc-ensure-changelog-version
    hc-readme-grep-nightly
    hc-build-release-artifacts

    hc-do-release

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
