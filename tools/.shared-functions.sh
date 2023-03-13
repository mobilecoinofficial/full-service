#!/bin/bash
# Copyright (c) 2018-2022 The MobileCoin Foundation
# Set of shared functions for full-service build, test and run tools.

# shellcheck disable=SC2034  # Allow unused vars in this shared file.

GIT_BASE=$(git rev-parse --show-toplevel)
AM_I_IN_MOB_PROMPT="no"
CARGO_TARGET_DIR="${GIT_BASE}/target"

# Assume that if you're git directory is /tmp/mobilenode that we're in mob prompt
if [[ "${GIT_BASE}" == "/tmp/mobilenode" ]]
then
    AM_I_IN_MOB_PROMPT="yes"
fi

if [[ "${AM_I_IN_MOB_PROMPT}" == "yes" ]]
then
    echo "I'm in mob prompt!"
    WORK_DIR="${WORK_DIR:-"${GIT_BASE}/.mob/${net}"}"
    LISTEN_ADDR="0.0.0.0"
else
    WORK_DIR="${WORK_DIR:-"${HOME}/.mobilecoin/${net}"}"
    LISTEN_ADDR="127.0.0.1"
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
