FROM holochain/holochain-rust:circle.09.warm.sodium

RUN nix-shell --run hc-test-hc
