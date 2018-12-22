# FROM holochain/holochain-rust:circle.09.warm.sodium
FROM holochain/holochain-rust:circle.02.warm.hdk

RUN nix-shell --run hc-test-hc
