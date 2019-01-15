FROM holochain/holochain-rust:circle.03.warm.tools

# the default command can be the unit tests, why not?
# if the build works properly this should be fast in incremental compilation
CMD nix-shell --run hc-test
