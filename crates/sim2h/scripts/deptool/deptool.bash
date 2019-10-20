#!/bin/bash

set -Eeuo pipefail

function _usage() {
  echo "holochain deptool - changing Cargo deps for testing"
  echo "usage: deptool [options] cmd"
  echo "commands:"
  echo "  lib3h - deptool lib3h <subcmd>"
  echo "    version - deptool lib3h version <version>"
  echo "    branch - deptool lib3h branch <branch-name>"
  echo "    path - deptool lib3h path <path>"
  echo "options:"
  echo "  -h --help: additional help for command"
  exit 1
}

function _lib3h_deps() {
  local __dep_str="${1}"
  echo "setting lib3h deps to ${__dep_str}"

  local __deps=$(find ../../crates -maxdepth 2 -mindepth 2 -name Cargo.toml)
  echo "${__deps}"
  sed -i'' "s/\\(lib3h[^[:space:]]*[[:space:]]\\+=[[:space:]]\\+\\).*/\\1${__dep_str//\//\\\/}/" ${__deps}
}

function _lib3h_path_deps() {
  local __l3h_path="$(readlink -f ${__pwd}/${1})"
  local __l3h_lib3h="{ path = \"${__l3h_path}/crates/lib3h\" }"
  local __l3h_proto="{ path = \"${__l3h_path}/crates/lib3h_protocol\" }"
  local __l3h_crypto="{ path = \"${__l3h_path}/crates/crypto_api\" }"
  local __l3h_sodium="{ path = \"${__l3h_path}/crates/sodium\" }"
  local __l3h_zombie_actor="{ path = \"${__l3h_path}/crates/zombie_actor\" }"
  echo "using ${__l3h_lib3h} ${__l3h_proto} ${__l3h_crypto} ${__l3h_sodium} ${__l3h_zombie_actor}"

  local __deps=$(find ../.. -maxdepth 2 -mindepth 2 -name Cargo.toml)
  echo "${__deps}"
  sed -i'' "s/\\(lib3h[[:space:]]\\+=[[:space:]]\\+\\).*/\\1${__l3h_lib3h//\//\\\/}/" ${__deps}
  sed -i'' "s/\\(lib3h_protocol[[:space:]]\\+=[[:space:]]\\+\\).*/\\1${__l3h_proto//\//\\\/}/" ${__deps}
  sed -i'' "s/\\(lib3h_crypto_api[[:space:]]\\+=[[:space:]]\\+\\).*/\\1${__l3h_crypto//\//\\\/}/" ${__deps}
  sed -i'' "s/\\(lib3h_sodium[[:space:]]\\+=[[:space:]]\\+\\).*/\\1${__l3h_sodium//\//\\\/}/" ${__deps}
  sed -i'' "s/\\(lib3h_zombie_actor[[:space:]]\\+=[[:space:]]\\+\\).*/\\1${__l3h_zombie_actor//\//\\\/}/" ${__deps}
}

function _cmd() {
  local __cmd="${1:-<unset>}"
  case "${__cmd}" in
    lib3h)
      local __sub="${2:-<unset>}"
      case "${__sub}" in
        version)
          if [ ${__help} == 1 ]; then
            echo "deptool lib3h version"
            echo " - set the various lib3h dep versions"
            echo " - example: deptool lib3h version 0.0.9"
            echo "   will set: lib3h = \"=0.0.9\""
            exit 1
          fi
          _lib3h_deps "\"=${3}\""
          ;;
        branch)
          if [ ${__help} == 1 ]; then
            echo "deptool lib3h branch"
            echo " - set the various lib3h dep to a github branch"
            echo " - example: deptool lib3h branch test-a"
            echo "   will set: lib3h = { git = \"https://github.com/holochain/lib3h\", branch = \"test-a\" }"
            exit 1
          fi
          _lib3h_deps "{ git = \"https://github.com/holochain/lib3h\", branch = \"${3}\" }"
          ;;
        path)
            if [ ${__help} == 1 ]; then
                echo "deptool lib3h path"
                echo " - set the various lib3h dep to a local file path"
                echo " - example: deptool lib3h path ../lib3h"
                echo "   will set: lib3h = { path = \"../lib3h/crates/...\" }"
                exit 1
            fi
            _lib3h_path_deps "${3}"
            ;;
        *)
          if [ ${__help} == 1 ]; then
            echo "deptool lib3h"
            echo " - alter lib3h dependencies in this repo"
            echo " - example: deptool lib3h version 0.0.9"
            echo " - example: deptool lib3h branch test-a"
            exit 1
          fi
          echo "unexpected lib3h subcommand '${__sub}'"
          _usage
          ;;
      esac
      ;;
    *)
      echo "unexpected command '${__cmd}'"
      _usage
      ;;
  esac
}

function _this_dir() {
  local __src_dir="${BASH_SOURCE[0]}"
  local __work_dir=""
  while [ -h "${__src_dir}" ]; do
    __work_dir="$(cd -P "$(dirname "${__src_dir}")" >/dev/null 2>&1 && pwd)"
    __src_dir="$(readlink "${__src_dir}")"
    [[ ${__src_dir} != /* ]] && __src_dir="${__work_dir}/${__src_dir}"
  done
  __work_dir="$(cd -P "$(dirname "${__src_dir}")" >/dev/null 2>&1 && pwd)"

  cd "${__work_dir}"
}

function main() {
  local __pwd="$(pwd)"

  _this_dir

  local __cmd=""
  local __help="0"
  while (( "${#}" )); do
    case "${1}" in
      -h|--help)
        __help="1"
        shift
        ;;
      --) # end argument parsing
        shift
        break
        ;;
      -*|--*=) # unsupported flags
        echo "Error: Unsupported option ${1}" >&2
        exit 1
        ;;
      *) # preserve positional arguments
        __cmd="$__cmd ${1}"
        shift
        ;;
    esac
  done

  _cmd ${__cmd}
}

main "${@}"
