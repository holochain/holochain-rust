FROM holochain/holochain-rust:develop

RUN rustup default stable
RUN cargo install mdbook --vers "^0.1.0"

WORKDIR /holochain/doc/holochain_101

# Port for web access
EXPOSE 3000
# Port for websocket (live reload)
EXPOSE 3001
