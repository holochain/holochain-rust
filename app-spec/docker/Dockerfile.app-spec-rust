FROM holochain/holochain-rust:develop

RUN export DEBIAN_FRONTEND=noninteractive

# readline needed for hcshell & propierties needed for add-apt-repository
RUN apt-get update && apt-get install --yes \
  libreadline6-dev \
  software-properties-common

# Install latest version of QT needed for hcshell
RUN add-apt-repository ppa:beineri/opt-qt-5.11.1-xenial
RUN apt-get update
RUN apt-get --yes install qt511-meta-full
RUN printf "/opt/qt511/bin\n/opt/qt511/lib" > /etc/xdg/qtchooser/default.conf

RUN rustup toolchain install nightly-x86_64-unknown-linux-gnu
RUN rustup toolchain install nightly-2018-07-17-x86_64-unknown-linux-gnu
RUN rustup default nightly
RUN rustup target add wasm32-unknown-unknown
WORKDIR /holochain

RUN git clone https://github.com/holochain/holochain-cmd
RUN git clone https://github.com/holochain/holosqape

ENV PATH "/holochain/holochain-cmd/target/release:/holochain/holosqape/hcshell:$PATH"

WORKDIR /holochain/holochain-cmd
RUN git submodule init
RUN git submodule update
RUN cargo update
RUN cargo +nightly build --release
WORKDIR /holochain/holosqape
RUN git submodule init
RUN git submodule update
WORKDIR /holochain/holosqape/holochain-rust
RUN cargo update
RUN cargo +$TOOLS_NIGHTLY build --release
WORKDIR /holochain/holosqape/bindings
RUN qmake
RUN make
WORKDIR /holochain/holosqape/hcshell
RUN qmake CONFIG+=release
RUN make

WORKDIR /holochain
RUN chown holochain:holochain -R /holochain/*
