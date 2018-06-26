FROM ubuntu:latest

ENV TARGET=x86_64-unknown-linux-musl
ENV BUILD_DIR=/src/target/x86_64-unknown-linux-musl/release/

RUN apt-get update && \
    apt-get install \
        curl \
        gcc \
        -y

RUN curl https://sh.rustup.rs -sSf -o /tmp/rustup-init.sh
RUN sh /tmp/rustup-init.sh -y

RUN ~/.cargo/bin/rustup target add ${TARGET}

ONBUILD COPY . /src
ONBUILD WORKDIR /src

ONBUILD RUN ~/.cargo/bin/cargo test --release --target=${TARGET}
ONBUILD RUN ~/.cargo/bin/cargo build --release --target=${TARGET}
