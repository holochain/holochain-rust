let
  pkgs = import ../../../../nixpkgs/nixpkgs.nix;
  release = import ../../../config.nix;

  name = "hc-release-docs-changelog-version-sync";

  template =
  ''
[Unreleased]

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security
'';

  changelog-path = "./CHANGELOG.md";
  unreleased-path = "./CHANGELOG-UNRELEASED.md";

  # cat ${unreleased-path} | sed "s/\[Unreleased\]/${template}\#\# \[${release.core.version.current}\] - $(date --iso --u)/"
  script = pkgs.writeShellScriptBin name
  ''
   echo
   echo "locking off changelog version"
   echo

   echo '${template}' > '${unreleased-path}'

   if ! $(grep -q "\[${release.core.version.current}\]" ${changelog-path})
    then
     echo "timestamping and retemplating changelog"
   fi
  '';
in
script
