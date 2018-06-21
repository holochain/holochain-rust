FROM alpine:edge

WORKDIR /root

# rustup/cargo
ENV RUSTUP_TOOLCHAIN=stable-x86_64-unknown-linux-musl
RUN cat /etc/apk/repositories
RUN echo '@testing http://dl-cdn.alpinelinux.org/alpine/edge/testing' >> /etc/apk/repositories && \
    # echo 'http://dl-cdn.alpinelinux.org/alpine/latest-stable/main' >> /etc/apk/repositories && \
    # echo 'http://dl-cdn.alpinelinux.org/alpine/latest-stable/community' >> /etc/apk/repositories && \
    apk update && \
    apk add --no-cache cargo@testing rust@testing \
        curl ca-certificates gcc musl-dev make zlib-dev openssl-dev perl git
RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly -y
# RUN git clone https://github.com/rust-lang-nursery/rustup.rs.git
# WORKDIR /root/rustup.rs
# RUN cargo build
# RUN sh ./rustup
# RUN git clone https://github.com/rust-lang-nursery/rustup.rs.git && cd rustup.rs && cargo build --release && mv target/release/rustup-init /tmp
# RUN sh /tmp/rustup-init
ENV PATH /root/.cargo/bin:$PATH
RUN rustc --version
RUN rustup default nightly

# install tarpaulin
RUN apk update && apk add openssl-dev pkgconfig cmake zlib-dev
RUN RUSTFLAGS="--cfg procmacro2_semver_exempt" cargo install cargo-tarpaulin

# do codecov
COPY . /root/holochain
WORKDIR /root/holochain
CMD cargo tarpaulin --all
