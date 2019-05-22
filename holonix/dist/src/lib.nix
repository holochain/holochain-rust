let
 pkgs = import ../../nixpkgs/nixpkgs.nix;
 rust = import ../../rust/config.nix;
 dist = import ../config.nix;
 git = import ../../git/config.nix;
in rec
{
 artifact-name = args: "${args.name}-v${dist.version}-${args.target}";

 artifact-url = args: "https://github.com/${git.github.repo}/releases/download/v${dist.version}/${artifact-name args}.tar.gz";

 binary-derivation = args:
  pkgs.stdenv.mkDerivation {
   name = "${args.binary}";

   src = pkgs.fetchurl {
    url = artifact-url ( { target = dist.artifact-target; } // args );
    sha256 = if pkgs.stdenv.isDarwin then args.sha256.darwin else args.sha256.linux;
   };

  unpackPhase = "tar --strip-components=1 -zxvf $src";

  installPhase =
  ''
    mkdir -p $out/bin
    mv ${args.binary} $out/bin/${args.binary}
  '';

  postFixup =
    if
      pkgs.stdenv.isDarwin
    then
      ''
      echo;
      ''
    else
      ''
        patchelf --set-interpreter "$(cat $NIX_CC/nix-support/dynamic-linker)" $out/bin/${args.binary}
        patchelf --shrink-rpath $out/bin/${args.binary}
      '';

  };

}
