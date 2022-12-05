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

# Check to see if leading argument starts with "--".
# If so execute with full-service for compatibility with the previous container/cli arg only configuration.
if [[ "${1}" =~ ^--.* ]]
then
    exec "/usr/local/bin/full-service" "$@"
else
    exec "$@"
fi
