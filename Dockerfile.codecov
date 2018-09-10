FROM holochain/holochain-rust:develop

RUN chown holochain:holochain /holochain
USER ${DOCKER_BUILD_USER}

# docker hub sets working directory to the directory containing the Dockerfile
COPY --chown=holochain . /holochain

WORKDIR /holochain

RUN ls -la

RUN make wasm_build

# warm cargo as much as possible
# we can't warm with a tarpaulin run as codecov requires runtime docker args
RUN cargo +${WASM_NIGHTLY} test
USER root
