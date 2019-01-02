FROM holochain/holochain-rust:circle.13.warm.net-ipc

RUN nix-shell --run hc-install-cmd

CMD nix-shell --run hc-install-cmd
