FROM nixos/nix
CMD nix-shell ./shell.core.nix --run hc-wasm-build
