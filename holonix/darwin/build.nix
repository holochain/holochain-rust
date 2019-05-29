let
  pkgs = import ../nixpkgs/nixpkgs.nix;

  # https://stackoverflow.com/questions/51161225/how-can-i-make-macos-frameworks-available-to-clang-in-a-nix-environment
  frameworks = if pkgs.stdenv.isDarwin then pkgs.darwin.apple_sdk.frameworks else {};

in
[]
++ pkgs.lib.optionals pkgs.stdenv.isDarwin [ frameworks.Security frameworks.CoreFoundation frameworks.CoreServices ]
