FROM holochain/holonix:latest

WORKDIR /holochain-rust/build

# get latest develop
ADD https://github.com/holochain/holochain-rust/archive/trycp.tar.gz /holochain-rust/build/develop.tar.gz
RUN tar --strip-components=1 -zxvf develop.tar.gz

RUN nix-shell --run hc-test
