#!/usr/bin/env bats

setup () {
 hc-cli-uninstall
}

teardown () {
 hc-cli-uninstall
}

@test "install and uninstall" {

 echo '# in the nix store by default' >&3

 run command -v hc
 [[ "$output" == /nix/store* ]]

 echo '# shadowed after an installation' >&3
 hc-cli-install

 run command -v hc
 [[ "$output" == "$CARGO_HOME/bin/hc" ]]

 echo '# back to the nix store after an uninstall' >&3
 hc-cli-uninstall

 run command -v hc
 [[ "$output" == /nix/store* ]]

}

@test "init, generate, test" {

 set -euo pipefail

 export USER=$(id -u -n)
 export app_name=my_first_app
 export zome_name=my_zome

 echo '# install local cli build' >&3

 hc-cli-install

 echo '# steps adapted from quickstart 2019-09-11' >&3

 echo '# hc init "$TMP/$app_name"' >&3
 run hc init "$TMP/$app_name"

 echo '# cd "$TMP/$app_name"' >&3
 cd "$TMP/$app_name"

 echo '# hc generate "zomes/$zome_name"' >&3
 run hc generate "zomes/$zome_name"

 echo '# hc test' >&3
 export CARGO_TARGET_DIR="$CARGO_TARGET_DIR/cli/hc-test"
 echo "# $CARGO_TARGET_DIR" >&3
 run hc test

}
