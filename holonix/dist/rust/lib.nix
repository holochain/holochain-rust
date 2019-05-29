let
 release = import ../../release/config.nix;
 rust = import ../../rust/config.nix;
 dist = import ../../dist/config.nix;
in
{

 build-rust-artifact = params:
 let
  artifact-name = "${params.path}-${release.core.version.current}-${dist.artifact-target}";
 in
 ''
  echo
  echo "building ${artifact-name}..."
  echo

  CARGO_INCREMENTAL=0 cargo rustc --manifest-path ${params.path}/Cargo.toml --target ${rust.generic-linux-target} --release -- -C lto
  mkdir -p ${dist.path}/${artifact-name}
  cp target/${rust.generic-linux-target}/release/${params.name} ${params.path}/LICENSE ${params.path}/README.md ${dist.path}/${artifact-name}
  tar -C ${dist.path}/${artifact-name} -czf ${dist.path}/${artifact-name}.tar.gz . && rm -rf ${dist.path}/${artifact-name}
 '';

}
