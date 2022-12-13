#!/bin/bash
# Copyright (c) 2018-2022 The MobileCoin Foundation
# Set of shared functions for full-service build, test and run tools.


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


check_xcode() {
  if [[ $(uname) == "Darwin" ]]; then
    xcode_version=$(xcodebuild -version | grep "Xcode" | awk '{print $2}' | sed 's/[.].*//')
    minimum_xcode_version=12
    if [[ $(printf '%s\n' "$xcode_version" "$minimum_xcode_version" | sort -V | head -n1) == "$xcode_version" ]]; then
      echo
    else
      echo "Xcode version $xcode_version is not supported. Use xcode-select to switch to $minimum_xcode_version."
      exit 1
    fi
  else
    echo 
  fi
}

