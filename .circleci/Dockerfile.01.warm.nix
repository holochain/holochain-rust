FROM holochain/holochain-rust:circle.00.start

# run a no-op to warm the nix store
RUN nix-shell --run "echo 1" --show-trace --max-jobs 4 --cores 0
