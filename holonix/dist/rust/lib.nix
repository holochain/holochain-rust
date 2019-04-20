let
 release = import ../../release/config.nix;
 rust = import ../../rust/config.nix;
 dist = import ../../dist/config.nix;
in
{

 build-rust-artifact = params:
 ''
  export artifact_name=`sed "s/unknown/generic/g" <<< "${params.path}-${release.core.version.current}-${rust.generic-linux-target}"`
  echo
  echo "building $artifact_name..."
  echo

  CARGO_INCREMENTAL=0 cargo rustc --manifest-path ${params.path}/Cargo.toml --target ${rust.generic-linux-target} --release -- -C lto
  mkdir -p ${dist.path}/$artifact_name
  cp target/${rust.generic-linux-target}/release/${params.name} ${params.path}/LICENSE ${params.path}/README.md ${dist.path}/$artifact_name
  tar -C ${dist.path}/$artifact_name -czf ${dist.path}/$artifact_name.tar.gz . && rm -rf ${dist.path}/$artifact_name
 '';

}
