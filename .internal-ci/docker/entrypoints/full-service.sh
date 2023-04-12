#!/bin/bash
# Copyright (c) 2018-2022 The MobileCoin Foundation
#
# Entrypoint script for full-service container

### CBB: docker with -it doesn't ctrl-c

set -e

echo "Starting Up with command ${1}"

# Make sure wallet and ledger directories exist.
mkdir -p "${MC_LEDGER_DB}"
mkdir -p "$(dirname "${MC_WALLET_DB}")"

# Remove unset wallet db value so we can run in Read Only mode.
if [[ -n "${MC_READ_ONLY}" ]]
then
    echo "MC_READ_ONLY set. Unset MC_WALLET_DB value."
    unset MC_WALLET_DB
fi

# Unset MC_PEER and TX_SOURCE_URL if we're running with a validator. The validator will need the consensus config.
if [[ -n "${MC_VALIDATOR}" ]]
then
    echo "MC_VALIDATOR set. Unset MC_PEER and MC_TX_SOURCE_URL."
    unset MC_PEER
    unset MC_TX_SOURCE_URL
fi

# Check to see if leading argument starts with "--".
# If so execute with full-service for compatibility with the previous container/cli arg only configuration.
if [[ "${1}" =~ ^--.* ]]
then
    exec "/usr/local/bin/full-service" "$@"
else
    exec "$@"
fi
