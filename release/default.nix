{ holonix, pkgs }:
let
  release = import ./config.nix;

  github = pkgs.callPackage ./github {
   release = release;
  };

  rust = pkgs.callPackage ./rust {
   release = release;
  };

  pulse = pkgs.callPackage ./pulse {
   holonix = holonix;
   release = release;
   github = github;
  };

  docs = pkgs.callPackage ./docs {
   release = release;
  };
in
release // {
 buildInputs = []

 ++ (pkgs.callPackage ./audit {
  release = release;
  pulse = pulse;
 }).buildInputs

 ++ (pkgs.callPackage ./branch {
  release = release;
  github = github;
 }).buildInputs

 ++ (pkgs.callPackage ./deploy {
  release = release;
  github = github;
 }).buildInputs

 ++ (pkgs.callPackage ./prepare { }).buildInputs
 ++ pulse.buildInputs
 ++ rust.buildInputs
 ++ docs.buildInputs
 ++ github.buildInputs
 ;
}
