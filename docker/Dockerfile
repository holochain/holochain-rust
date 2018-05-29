FROM phusion/baseimage

# UID argument specified by docker build --build-arg uid=<uid> and defaults to 1000
ARG uid=${UID_MIN:-1000}

RUN useradd -mUu $uid holochain && adduser holochain sudo

# This removes some warning when installing packages when there is no X
ENV DEBIAN_FRONTEND noninteractive

# Get all the basic packages we need.
RUN apt-get update -qq -y && \
  apt-get install --no-install-recommends --yes \
    build-essential git-core \
    moreutils curl unzip wget bash bash-completion mc htop tzdata rsync nano less \
    zlib1g-dev libssl-dev libreadline-dev \
    cargo

COPY . /holochain
RUN chown -R holochain:holochain /holochain

USER holochain
EXPOSE 3141
