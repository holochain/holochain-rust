FROM ubuntu

WORKDIR /root

RUN apt-get update && apt-get install libssl-dev pkg-config cmake zlib1g-dev curl -y

RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly -y

ENV PATH /root/.cargo/bin:$PATH

RUN rustc --version

COPY . /root/holochain

WORKDIR /root/holochain

#
# RUN RUSTFLAGS="--cfg procmacro2_semver_exempt" cargo install cargo-tarpaulin
