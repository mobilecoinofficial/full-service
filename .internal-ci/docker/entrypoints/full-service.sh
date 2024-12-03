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

# Restore from existing ledger database.
if [[ -n "${MC_LEDGER_DB_URL}" ]]
then
    echo "MC_LEDGER_DB_URL set, restoring ${MC_LEDGER_DB}/data.mdb from backup"
    if [[ -f "${MC_LEDGER_DB}/data.mdb" ]]
    then
        echo "${MC_LEDGER_DB}/data.mdb exists. Skipping restore"
    else
        echo "Restoring from ${MC_LEDGER_DB_URL}"
        curl -L "${MC_LEDGER_DB_URL}" -o "${MC_LEDGER_DB}/data.mdb"
    fi
fi

# Run until we have a valid ledger database then quit.
if [[ -n "${SYNC_LEDGER_ONLY}" ]]
then
    echo "SYNC_LEDGER_ONLY set. Exiting after syncing ledger."
    RUST_LOG=error /usr/local/bin/full-service &
    /usr/local/bin/wait-for-full-service.sh
    exit 0
fi

# Check to see if leading argument starts with "--".
# If so execute with full-service for compatibility with the previous container/cli arg only configuration.
if [[ "${1}" =~ ^--.* ]]
then
    exec "/usr/local/bin/full-service" "$@"
else
    exec "$@"
fi
