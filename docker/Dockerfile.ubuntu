# WARNING, this file must be updated in two places: the repo root, and docker subdirectory.

FROM ubuntu:xenial

# This removes some warning when installing packages when there is no X
ENV DEBIAN_FRONTEND noninteractive

RUN apt-get update && apt-get install --yes\
  libssl-dev \
  pkg-config \
  cmake\
  zlib1g-dev \
  curl \
  qt5-default \
  python2.7 \
  gosu \
  git

# install nodejs
RUN curl -sL https://deb.nodesource.com/setup_11.x | bash
RUN apt-get install -y nodejs

# Set the internal user for the docker container (non-root)
ENV DOCKER_BUILD_USER holochain

RUN useradd -ms /bin/bash ${DOCKER_BUILD_USER}
USER ${DOCKER_BUILD_USER}

ADD ./Makefile ./Makefile

# need to set path manually in docker as normal `export` in make doesn't work
ENV PATH /home/${DOCKER_BUILD_USER}/.cargo/bin:$PATH
RUN make install_rustup

RUN make ensure_wasm_target

RUN make install_rust_tools

RUN make install_ci

WORKDIR /holochain

USER root
