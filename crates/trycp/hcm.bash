#!/bin/bash

#set -xv

__root_dir="$(pwd)"
__db_dir="${__root_dir}/db"
mkdir -p "$__db_dir"

function _usage() {
    echo "holochain conductor manger"
    echo "usage: hcm [options] cmd"
    echo "commands:"
    echo "  player - trcyp player <player-id> <config-file>"
    echo "  spawn - hcm spawn <player-id>"
    echo "  kill - hcm kill <player-id>"
    echo "options:"
    echo "  -h --help: additional help for command"
    exit 1
}

function _setup_player() {
    local id="${1}";
    if [ $id == "<unset>" ]; then
        echo "expecting player id as the first argument"
        exit 1
    fi
    player_config="${__db_dir}/${id}.toml"
}

function _check_player() {
    local id="${1}";
    _setup_player "$id"
    if [ ! -f $player_config ]; then
        echo "player config not setup"
        exit 1
    fi
    pid_file="${__db_dir}/${id}.pid"
    if [ -f $pid_file ]; then
        conductor_pid=$(<$pid_file)
    else
        conductor_pid=""
    fi
}

function _player() {
    local id="${1}";
    local config_src="${2}";
    if [ $config_src == "<unset>" ]; then
        echo "expecting config_file as second argument"
        exit 1
    fi
    _setup_player "$id"
    if [ -f $player_config ]; then
        echo "player config already exists"
        exit 1
    else
        cp $config_src $player_config
    fi
}

function _spawn() {
    local id="${1}";
    _check_player "$id"
    if ps -p $conductor_pid &> /dev/null
    then
        echo "$id is already running"
        exit 1
    fi
    echo "starting conductor"
    ping google.com > /dev/null &
    echo "$!" > $pid_file
}

function _kill() {
    local id="${1}";
    _check_player "$id"
    rm $pid_file
    if ps -p $conductor_pid &> /dev/null
    then
        kill $conductor_pid
        echo "conductor stopped"

    else
        echo "conductor not running at $id"
        exit 1
    fi
}
function _cmd() {
  local __cmd="${1:-<unset>}"
  case "${__cmd}" in
      player)
          if [ ${__help} == 1 ]; then
              echo "trcyp player <player-id> <config>"
              exit 1
          fi
          _player "${2:-<unset>}" "${3:-<unset>}"
          ;;
      spawn)
          if [ ${__help} == 1 ]; then
              echo "trcyp spawn <player-id>"
              exit 1
          fi
          _spawn "${2:-<unset>}"
          ;;
      kill)
          if [ ${__help} == 1 ]; then
              echo "trcyp kill <player-id>"
              exit 1
          fi
          _kill "${2:-<unset>}"
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
