#!/bin/bash

set -e

WALLETS_JSON=${WALLETS_JSON:-"/secrets/wallets.json"}
FULL_SERVICE_URL=${FULL_SERVICE_URL:-"http://127.0.0.1:9090/wallet/v2"}

fs_request()
{
    curl --connect-timeout 2 -fsSL "${FULL_SERVICE_URL}" -X POST -H 'Content-type: application/json' --data "${1}" 2>&1
}

# These would have been cleaner as a heredoc, but heredoc requires a read-write tmp fs space.
# get_accounts payload
get_accounts()
{
    echo '{ "method": "get_accounts", "params": {}, "jsonrpc": "2.0", "id": 1 }'
}

# import_account payload
import_account()
{
    echo "{ \"method\": \"import_account\", \"params\": { \"name\": \"${1}\", \"mnemonic\": \"${2}\", \"key_derivation_version\": \"2\" }, \"jsonrpc\": \"2.0\",\"id\": 1 }"
}

log()
{
    echo "{ \"app\": \"${0}\", \"message\": \"${1}\" }"
}

log "Starting ${0}"

if [[ -f "${WALLETS_JSON}" ]]
then
    log "Wait for full-service to come up"
    while ! fs_request "$(get_accounts)" >/dev/null 2>&1
    do
        log "Waiting for full-service"
        sleep 10
    done

    # iterate through wallets and import
    # importing accounts doesn't throw an http error if they already exisit,
    # so this is safe to run on each startup.
    accounts=$(jq -r '.accounts[].name' "${WALLETS_JSON}")
    for name in ${accounts}
    do
        log "Importing account ${name}"
        mnemonic=$(jq -r ".accounts[] | select(.name==\"${name}\").mnemonic" "${WALLETS_JSON}")
        fs_request "$(import_account "${name}" "${mnemonic}")" >/dev/null
    done
else
    log "${WALLETS_JSON} not found - no wallets to import."
fi

log "${0} is finished"
