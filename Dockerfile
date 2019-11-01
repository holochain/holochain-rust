FROM holochain/holonix:latest

WORKDIR /holochain-rust/build
COPY . .

RUN nix-shell --run hc-test