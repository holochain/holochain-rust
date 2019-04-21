let
  pkgs = import ../../../../nixpkgs/nixpkgs.nix;
  release = import ../../../config.nix;

  name = "hc-release-docs-changelog-sync-version";

  template =
  ''\
\[Unreleased\]\n\n\
\#\#\# Added\n\n\
\#\#\# Changed\n\n\
\#\#\# Deprecated\n\n\
\#\#\# Removed\n\n\
\#\#\# Fixed\n\n\
\#\#\# Security\n\n\
'';

  script = pkgs.writeShellScriptBin name
  ''
   echo
   echo "locking off changelog version"
   echo

   if ! $(grep -q "\[${release.core.version.current}\]" ./CHANGELOG.md)
    then
     echo "timestamping and retemplating changelog"
     sed -i "s/\[Unreleased\]/${template}\#\# \[${release.core.version.current}\] - $(date --iso --u)/" ./CHANGELOG.md
   fi
  '';
in
script
