{ pkgs }:
let
  config = import ./nix/config.nix;
  github = pkgs.callPackage ./github {
   release = config;
  };

  pulse = pkgs.callPackage ./pulse {
   release = config;
   github = github;
  };

  rust = pkgs.callPackage ./rust {
   release = config;
  };

  docs = pkgs.callPackage ./docs {
   release = config;
  };
in
{
 buildInputs = [
  (pkgs.callPackage ./nix/audit.nix {
   config = config;
   pulse = pulse;
   })

   (pkgs.callPackage ./nix/branch.nix {
    release = config;
    github = github;
   })

   (pkgs.callPackage ./nix/deploy.nix {
    release = config;
    github = github;
   })
 ]
 ++ pulse.buildInputs
 ++ rust.buildInputs
 ++ docs.buildInputs
 ++ github.buildInputs
 ;

 config = config;
}
