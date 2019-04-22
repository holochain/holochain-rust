let
 pkgs = import ../../nixpkgs/nixpkgs.nix;
 rust = import ../../rust/config.nix;
 dist = import ../config.nix;
 git = import ../../git/config.nix;
in
{
 binary-derivation = args:
  let
   artifact-name = "${args.name}-v${dist.version}-${dist.artifact-target}";
  in
  pkgs.stdenv.mkDerivation {
   name = "holochain-${args.name}";


   src = pkgs.fetchurl {
    url = "https://github.com/${git.github.repo}/releases/download/v${dist.version}/${artifact-name}.tar.gz";
    sha256 = args.sha256;
   };

   unpackPhase = ":";

   installPhase = ''
     mkdir -p $out/bin
     cp $src $out/bin/${args.binary}.tar.gz
     tar --strip-components=1 -C $out/bin -zxvf $out/bin/${args.binary}.tar.gz
     rm $out/bin/${args.binary}.tar.gz
     chmod +x $out/bin/${args.binary}
   '';
  };

}
