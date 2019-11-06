#!/bin/bash

# this script copies the websocket code over from lib3h
# making some changes to support running it in holochain-rust
# would be nice to just directly use the lib3h version at some point

function usage() {
  echo "usage: ./migrate-websocket.bash /path/to/lib3h"
  exit 1
}

function relocate() {
  local _src_dir="${BASH_SOURCE[0]}"
  local _work_dir=""
  while [ -h "${src_dir}" ]; do
    _work_dir="$(cd -P "$(dirname "${_src_dir}")" >/dev/null 2>&1 && pwd)"
    _src_dir="$(readlink "${_src_dir}")"
    [[ ${src_dir} != /* ]] && _src_dir="${_work_dir}/${_src_dir}"
  done
  work_dir="$(cd -P "$(dirname "${_src_dir}")" >/dev/null 2>&1 && pwd)"

  cd "${_work_dir}"
}

function edit_mod() {
  sed -z -i'' 's/'\
'pub mod actor;\n'\
'/'\
'extern crate env_logger;\n'\
'extern crate log;\n'\
'\n'\
'/' ./src/websocket/mod.rs
  sed -i'' 's/use crate::transport::[^;]\+;/'\
'use lib3h::transport::error::TransportResult;'\
'/' ./src/websocket/mod.rs
  sed -i'' 's/uri::Lib3hUri;/'\
'uri::Lib3hUri;\nuse wss_info::WssInfo;'\
'/' ./src/websocket/mod.rs
}

function edit_streams() {
  sed -z -i'' 's/'\
'use crate::transport::{\n'\
'    error::{TransportError, TransportResult},\n'\
'    websocket::{\n'\
'        tls::TlsConfig, wss_info::WssInfo, BaseStream, SocketMap, TlsConnectResult,\n'\
'        TlsMidHandshake, TlsSrvMidHandshake, TlsStream, WsConnectResult, WsMidHandshake,\n'\
'        WsSrvAcceptResult, WsSrvMidHandshake, WsStream, WssConnectResult, WssMidHandshake,\n'\
'        WssSrvAcceptResult, WssSrvMidHandshake, WssStream,\n'\
'    },\n'\
'};'\
'/'\
'use crate::websocket::{\n'\
'    tls::TlsConfig, wss_info::WssInfo, BaseStream, SocketMap, TlsConnectResult, TlsMidHandshake,\n'\
'    TlsSrvMidHandshake, TlsStream, WsConnectResult, WsMidHandshake, WsSrvAcceptResult,\n'\
'    WsSrvMidHandshake, WsStream, WssConnectResult, WssMidHandshake, WssSrvAcceptResult,\n'\
'    WssSrvMidHandshake, WssStream,\n'\
'};\n'\
'use log::*;\n'\
'\n'\
'use lib3h::transport::error::{TransportError, TransportResult};\n'\
'/' ./src/websocket/streams.rs
  sed -i'' 's/sync::{Arc, Mutex}/sync::Arc/g' ./src/websocket/streams.rs
}

function edit_tcp() {
  sed -z -i'' 's/'\
'\nuse crate::transport::{\n'\
'    error::{ErrorKind, TransportError, TransportResult},\n'\
'    websocket::{\n'\
'        streams::{Acceptor, Bind, StreamManager},\n'\
'        tls::TlsConfig,\n'\
'        wss_info::WssInfo,\n'\
'    },\n'\
'};\n'\
'/'\
'use lib3h::transport::error::{ErrorKind, TransportError, TransportResult};\n'\
'\n'\
'use crate::websocket::{\n'\
'    streams::{Acceptor, Bind, StreamManager},\n'\
'    tls::TlsConfig,\n'\
'    wss_info::WssInfo,\n'\
'};\n'\
'use log::*;\n'\
'/' ./src/websocket/tcp.rs
}

function edit_tls() {
  sed -z -i'' 's/'\
'use crate::transport::{\n'\
'    error::TransportResult,\n'\
'    websocket::{FAKE_PASS, FAKE_PKCS12},\n'\
'};'\
'/'\
'use crate::websocket::{FAKE_PASS, FAKE_PKCS12};\n'\
'\n'\
'use lib3h::transport::error::TransportResult;'\
'/' ./src/websocket/tls.rs
  sed -i'' 's/crate::transport::websocket/crate::websocket/g' ./src/websocket/tls.rs
}

function edit_wss_info() {
  sed -i'' 's/crate::transport::websocket/crate::websocket/g' ./src/websocket/wss_info.rs
}

function main() {
  local _pwd="$(pwd)"
  local _l3h="$(readlink -f ${_pwd}/${1})/crates/lib3h/src/transport/websocket"
  relocate
  if [ ! -d "${_l3h}" ]; then
    usage
  fi
  cp -a "${_l3h}" ./src/
  rm -f ./src/websocket/actor.rs
  edit_mod
  edit_streams
  edit_tcp
  edit_tls
  edit_wss_info
}

main "${@}"
