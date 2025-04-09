#!/bin/bash
# Copyright (c) 2018-2022 The MobileCoin Foundation
# Set of shared functions for full-service build, test and run tools.

GIT_BASE=$(git rev-parse --show-toplevel)
AM_I_IN_MOB_PROMPT="no"

# Assume that if you're git directory is /tmp/mobilenode that we're in mob prompt
if [[ "${GIT_BASE}" == "/tmp/mobilenode" || "${GIT_BASE}" == "/workspaces/full-service" ]]
then
    AM_I_IN_MOB_PROMPT="yes"
fi

if [[ -z "${net}" ]]
then
    echo "ERROR: <network> is not set"
    exit 1
fi

if [[ "${AM_I_IN_MOB_PROMPT}" == "yes" ]]
then
    # Set cargo target dir to include the "net"
    CARGO_TARGET_DIR="${GIT_BASE}/target/docker/${net}"
    # CBB: Deprecate the concept of "workdir" to simplify paths/scripts.
    #      This is now a symlink to RELEASE_DIR
    WORK_DIR=${WORK_DIR:-"${GIT_BASE}/.mob/${net}"}
    LISTEN_ADDR="0.0.0.0"
else
    CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-"${GIT_BASE}/target/${net}"}
    WORK_DIR=${WORK_DIR:-"${GIT_BASE}/.mob/${net}"}
    LISTEN_ADDR="127.0.0.1"
fi

RELEASE_DIR=${CARGO_TARGET_DIR}/release
export CARGO_TARGET_DIR RELEASE_DIR WORK_DIR LISTEN_ADDR

# Setup release dir - set in .shared-functions.sh
mkdir -p "${RELEASE_DIR}"

# Setup wallet dbs.  Don't put them in target so they don't get cleaned up by cargo clean.
WALLET_DB_DIR="${WALLET_DB_DIR:-".mob/${net}-db/wallet-db"}"
LEDGER_DB_DIR="${LEDGER_DB_DIR:-".mob/${net}-db/ledger-db"}"
mkdir -p "${WALLET_DB_DIR}"
mkdir -p "${LEDGER_DB_DIR}"

if [[ "${AM_I_IN_MOB_PROMPT}" == "yes" ]]
then
    # migrate wallet/ledger db to release_dir and remove workdir to make room
    # for the symlink
    if [[ ! -L "${WORK_DIR}" && -d "${WORK_DIR}" ]]
    then
        if [[ -d "${WORK_DIR}/wallet-db" ]]
        then
            mv "${WORK_DIR}/wallet-db" "${WALLET_DB_DIR}"
        fi
        if [[ -d "${WORK_DIR}/ledger-db" ]]
        then
            mv "${WORK_DIR}/ledger-db" "${LEDGER_DB_DIR}"
        fi
        rm -rf "${WORK_DIR}"
    fi

    # create parent workdir and link release_dir to work_dir
    mkdir -p "$(dirname "${WORK_DIR}")"

    # At this point WORK_DIR can only be a symlink, remove it and create a new one.
    rm -f "${WORK_DIR}"

    # this needs to be a relative link from GIT_BASE
    ln -s -r "${RELEASE_DIR}" "${WORK_DIR}"
fi


# debug - echo a debug message
#  1: message
debug()
{
    msg="${1}"

    if [[ -n "${RUNNER_DEBUG}" ]]
    then
        echo "::debug::${msg}"
    elif [[ -n "${DEBUG}" ]]
    then
        echo "DEBUG: ${msg}" >&2
    fi
}

# get_css_file - download a specified enclave measurement css file
#  1: css file name - consensus-enclave.css, ingest-enclave.css
get_css_file()
{
    net="${1}"
    css_file="${2}"

    # Default Variables - you can override these with environment var settings.
    CSS_BASE_URL=${CSS_BASE_URL:-"https://enclave-distribution.${net}.mobilecoin.com"}
    CSS_JSON_FILE=${CSS_JSON_FILE:-"production.json"}

    # Get remote css
    debug "  Pulling index file from ${CSS_BASE_URL}/${CSS_JSON_FILE}"
    json=$(curl -fsSL --retry 3 "${CSS_BASE_URL}/${CSS_JSON_FILE}")

    # Extract url - could we install jq?
    css_file_base=$(basename "${css_file}")
    css_url=$(echo "${json}" | grep "${css_file_base}" | awk '{print $2}' | tr -d \" | tr -d ,)

    debug "  Pulling css file from ${CSS_BASE_URL}/${css_url}"
    curl -fsSL --retry 3 "${CSS_BASE_URL}/${css_url}" -o "${css_file}"

    debug "  css file saved ${css_file}"
    echo "${css_file}"
}

# 1: pid file to check
check_pid_file()
{
    if [[ -f "${1}" ]]
    then
        pid=$(cat "${1}")
        if ps -p "${pid}" > /dev/null
        then
            echo "running"
        else
            echo "not running"
        fi
    fi
}

function parse_url()
{
    # 1: url
    # 2: variable name to store the result
    local varname=$2
    debug "varname: $varname"

    local proto full_url user hostport host port path
    # extract the protocol
    proto=$(echo "$1" | grep :// | sed -e's,^\(.*://\).*,\1,g')
    debug "proto: $proto"
    # remove the protocol
    full_url="${1/$proto/}"
    debug "url: $full_url"
    # extract the user (if any)
    if [[ "${full_url}" =~ '@' ]]
    then
        user=$(echo "${full_url}" | cut -d@ -f1)
    else
        user=""
    fi
    debug "user: $user"
    # extract the host and port
    hostport=$(echo "${full_url/$user@/}" | cut -d/ -f1)
    debug "host and port: $hostport"
    # by request host without port
    host="${hostport/:*/}"
    debug "host: $host"
    # by request - try to extract the port
    port=$(echo "${hostport}" | sed -e 's,^.*:,:,g' -e 's,.*:\([0-9]*\).*,\1,g' -e 's,[^0-9],,g')
    debug "port: $port"
    # extract the path (if any)
    path=$(echo "${full_url}" | grep / | cut -d/ -f2-)
    debug "path: $path"

    # shellcheck disable=SC1087 # this is a dynamic variable name doen't seem to work with brackets.
    declare -g "$varname[proto]=${proto}"
    # shellcheck disable=SC1087
    declare -g "$varname[url]=${full_url}"
    # shellcheck disable=SC1087
    declare -g "$varname[user]=${user}"
    # shellcheck disable=SC1087
    declare -g "$varname[hostport]=${hostport}"
    # shellcheck disable=SC1087
    declare -g "$varname[port]=${port}"
    # shellcheck disable=SC1087
    declare -g "$varname[host]=${host}"
    # shellcheck disable=SC1087
    declare -g "$varname[path]=${path}"
}
