FROM frolvlad/alpine-glibc

WORKDIR /root

# rustup/cargo
# ENV RUSTUP_TOOLCHAIN=nightly-x86_64-unknown-linux-musl
RUN cat /etc/apk/repositories
RUN echo '@testing http://dl-cdn.alpinelinux.org/alpine/edge/testing' >> /etc/apk/repositories

RUN apk update
RUN apk upgrade
RUN apk add curl ca-certificates gcc musl-dev make zlib-dev openssl-dev
RUN apk add cargo rust

ENV PATH /root/.cargo/bin:$PATH

RUN apk del cargo rust
# RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly-x86_64-unknown-linux-musl -y
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

RUN rustc --version

# install tarpaulin
RUN apk add openssl-dev pkgconfig cmake zlib-dev
# RUN RUSTFLAGS="--cfg procmacro2_semver_exempt" cargo install cargo-tarpaulin

# do codecov
# COPY . /root/holochain
# WORKDIR /root/holochain
# CMD cargo tarpaulin --all
