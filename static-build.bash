#!/bin/bash

# This is an unsupported tool for creating static linux builds for maximum distro compatibility
# we use this in the release of Holoscape.

# This script depends upon the following packages:
#  - some standard utilities
#  - docker
#  - binfmt_support
#  - qemu-user-static

# Run like `TGT_ARCH=x64 ./build-hc.bash`

# Adjust the following versions as desired or necessary (qemu download changes occasionally):

HC_VER="${HC_VER:-0.0.42-alpha2}"
HC_URL="https://github.com/holochain/holochain-rust/archive/v${HC_VER}.tar.gz"
HC_DIR="holochain-rust-${HC_VER}"

SSL_VER="${SSL_VER:-OpenSSL_1_1_1d}"
SSL_URL="https://github.com/openssl/openssl/archive/${SSL_VER}.tar.gz"
SSL_DIR="/openssl-${SSL_VER}"

QEMU_VER="qemu-user-static_3.1+dfsg-8+deb10u3_amd64"
QEMU_URL="http://ftp.us.debian.org/debian/pool/main/q/qemu/${QEMU_VER}.deb"

TGT_ARCH="${TGT_ARCH:-x64}"

qemu_bin=""
docker_from=""

case "${TGT_ARCH}" in
  "x32")
    qemu_bin="qemu-i386-static"
    docker_from="i386/debian:jessie-slim"
    echo "TGT_ARCH x32 coming soon"
    exit 1
    ;;
  "x64")
    qemu_bin="qemu-x86_64-static"
    docker_from="amd64/debian:jessie-slim"
    ;;
  "arm32")
    qemu_bin="qemu-arm-static"
    docker_from="arm32v7/debian:jessie-slim"
    echo "TGT_ARCH arm32 coming soon"
    exit 1
    ;;
  "arm64")
    qemu_bin="qemu-aarch64-static"
    docker_from="arm64v8/debian:jessie-slim"
    echo "TGT_ARCH arm64 coming soon"
    exit 1
    ;;
  *)
    echo "UNSUPPORTED ARCHITECTURE: '${TGT_ARCH}' - must be: x32, x64, arm32, or arm64"
    exit 1
    ;;
esac

img="holochain-v${HC_VER}-${TGT_ARCH}"

echo "building ${img} binaries"

if [ ! -f "${QEMU_VER}.deb" ]; then
  curl -L -O "${QEMU_URL}"
fi
( \
  mkdir -p ./qemu && \
  cd ./qemu && \
  ar x "../${QEMU_VER}.deb" && \
  tar xf data.tar.xz \
)

mkdir -p "${TGT_ARCH}"
cp -a -f "./qemu/usr/bin/${qemu_bin}" "${TGT_ARCH}"

# use a really old system on purpose so we bind to a widely portable libc ABI
# but we need to use a newer static openssl for security
cat > "${TGT_ARCH}/Dockerfile" <<EOF
FROM ${docker_from}
COPY ./${qemu_bin} /usr/bin/${qemu_bin}
# -- update the system -- #
RUN printf \
"deb http://archive.debian.org/debian/ jessie main\n"\
"deb-src http://archive.debian.org/debian/ jessie main\n"\
"\n"\
"deb http://security.debian.org jessie/updates main\n"\
"deb-src http://security.debian.org jessie/updates main\n"\
 > /etc/apt/sources.list
RUN apt-get update
RUN apt-get install -y curl ca-certificates build-essential git pkg-config
# -- build newer openssl -- #
RUN curl -L -O "${SSL_URL}"
RUN tar xf "${SSL_VER}.tar.gz"
RUN ( cd ${SSL_DIR} && ./config )
RUN ( cd ${SSL_DIR} && make -j$(nproc) )
ENV OPENSSL_STATIC="1"
ENV OPENSSL_LIB_DIR="${SSL_DIR}"
ENV OPENSSL_INCLUDE_DIR="${SSL_DIR}/include"
# -- download / configure rust -- #
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly-2019-11-16
ENV CARGO_BUILD_JOBS="$(nproc)"
ENV CARGO_INCREMENTAL="1"
ENV NUM_JOBS="$(nproc)"
ENV RUSTFLAGS="-D warnings -Z external-macro-backtrace -Z thinlto -C codegen-units=10"
# -- download / build holochain binaries -- #
RUN curl -L -O "${HC_URL}"
RUN tar xf "v${HC_VER}.tar.gz"
RUN ( cd ${HC_DIR} && PATH=/root/.cargo/bin:\$PATH cargo build --release -p hc )
RUN ( cd ${HC_DIR} && PATH=/root/.cargo/bin:\$PATH cargo build --release -p holochain )
RUN cp ${HC_DIR}/target/release/hc .
RUN cp ${HC_DIR}/target/release/holochain .
RUN ldd hc holochain
EOF

( cd ${TGT_ARCH} && docker build -t "${img}" . )
ID="$(docker create "${img}")"
docker cp "${ID}:hc" "./${TGT_ARCH}/hc-v${HC_VER}-${TGT_ARCH}"
docker cp "${ID}:holochain" "./${TGT_ARCH}/holochain-v${HC_VER}-${TGT_ARCH}"
docker rm "${ID}"
