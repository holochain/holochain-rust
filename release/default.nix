{ holonix, pkgs }:
let
  release = import ./config.nix;

  github = pkgs.callPackage ./github {
   holonix = holonix;
   release = release;
  };

  rust = pkgs.callPackage ./rust {
   release = release;
  };

  docs = pkgs.callPackage ./docs {
   release = release;
  };
in
release // {
 buildInputs = []

 ++ (pkgs.callPackage ./audit {
  release = release;
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
 ++ rust.buildInputs
 ++ docs.buildInputs
 ++ github.buildInputs
 ;
}
