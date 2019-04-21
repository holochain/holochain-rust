let

  pkgs = import ./holonix/nixpkgs/nixpkgs.nix;
  rust = import ./holonix/rust/config.nix;
  release = import ./holonix/release/config.nix;
  git = import ./holonix/git/config.nix;

  hc-test-app-spec = pkgs.writeShellScriptBin "hc-test-app-spec" "cd app_spec && . build_and_test.sh";

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

in
with pkgs;
stdenv.mkDerivation rec {
  name = "holochain-rust-environment";

  buildInputs = [

    # I forgot what these are for!
    # Reinstate and organise them ᕙ༼*◕_◕*༽ᕤ
    # coreutils
    # python

    hc-test-app-spec

    hc-test-all
    hc-codecov

    hc-prepare-crate-versions

    hc-ensure-changelog-version
    hc-readme-grep-nightly

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
