let
  pkgs = import ../../../../nixpkgs/nixpkgs.nix;
  release = import ../../../config.nix;

  name = "hc-release-docs-changelog-version-sync";

  heading-placeholder = "{{ version-heading }}";

  preamble =
  ''
# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
  '';

  template =
  ''
${preamble}
${heading-placeholder}

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

   template="$(cat ${unreleased-path})"
   heading_placeholder="${heading-placeholder}"
   heading="## [${release.core.version.current}] - $(date --iso --u)"

   echo $template
   echo $heading_placeholder
   echo $heading

   prepend=''${template/$heading_placeholder/$heading}
   current=$(cat ${changelog-path})

   printf '%s\n\n%s\n' "$prepend" "$current" > ${changelog-path}

   echo '${template}' > '${unreleased-path}'

   if ! $(grep -q "\[${release.core.version.current}\]" ${changelog-path})
    then
     echo "timestamping and retemplating changelog"
   fi
  '';
in
script
