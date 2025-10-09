#!/bin/bash

set -e -o pipefail

usage()
{
    echo "Usage:"
    echo "${0} <network>|check"
    echo "    <network> - main|prod|test"
    echo "Environment Variables:"
    echo "    FUNDING_FS_URL - optional: URL of the funding full-service"
    echo "    FUNDING_ACCOUNT_ID - optional: account_id of the funding account"
    echo "    TARGET_FS_URL - optional: URL of the full-service we want to test"
}

while (( "$#" ))
do
    case "${1}" in
        --help | -h)
            usage
            exit 0
            ;;
        *)
            net="${1}"
            shift 1
            ;;
    esac
done

if [[ -z "${net}" ]]
then
    echo "ERROR: <network> is not set"
    usage
    exit 1
fi

# use main instead of legacy prod
if [[ "${net}" == "prod" ]]
then
    echo "Detected \"prod\" legacy network setting. Using \"main\" instead."
    net=main
fi

# Grab current location and source the shared functions.
location=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
# shellcheck source=/dev/null
source "${location}/.shared-functions.sh"

case "${net}" in
    test)
        export MC_FOG_REPORT_URL="fog://fog.test.mobilecoin.com"
        export MC_WALLET_FILE=${MC_WALLET_FILE:-${GIT_BASE}/.mob/testnet_secret_mnemonic.json}
        export MC_FOG_AUTHORITY_SPKI="MIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEAvnB9wTbTOT5uoizRYaYbw7XIEkInl8E7MGOAQj+xnC+F1rIXiCnc/t1+5IIWjbRGhWzo7RAwI5sRajn2sT4rRn9NXbOzZMvIqE4hmhmEzy1YQNDnfALAWNQ+WBbYGW+Vqm3IlQvAFFjVN1YYIdYhbLjAPdkgeVsWfcLDforHn6rR3QBZYZIlSBQSKRMY/tywTxeTCvK2zWcS0kbbFPtBcVth7VFFVPAZXhPi9yy1AvnldO6n7KLiupVmojlEMtv4FQkk604nal+j/dOplTATV8a9AJBbPRBZ/yQg57EG2Y2MRiHOQifJx0S5VbNyMm9bkS8TD7Goi59aCW6OT1gyeotWwLg60JRZTfyJ7lYWBSOzh0OnaCytRpSWtNZ6barPUeOnftbnJtE8rFhF7M4F66et0LI/cuvXYecwVwykovEVBKRF4HOK9GgSm17mQMtzrD7c558TbaucOWabYR04uhdAc3s10MkuONWG0wIQhgIChYVAGnFLvSpp2/aQEq3xrRSETxsixUIjsZyWWROkuA0IFnc8d7AmcnUBvRW7FT/5thWyk5agdYUGZ+7C1o69ihR1YxmoGh69fLMPIEOhYh572+3ckgl2SaV4uo9Gvkz8MMGRBcMIMlRirSwhCfozV2RyT5Wn1NgPpyc8zJL7QdOhL7Qxb+5WjnCVrQYHI2cCAwEAAQ=="
        funding_account_id="2e181bc5ec273f2385439eecfacf967525c9d61003cc7c178ba4eaad84d1ac72"
        funding_fs_port="9091"
        port_forward_cmd="kubectl -n dev-wallet-testnet port-forward svc/full-service 9091:9090"
    ;;
    main)
        export MC_FOG_REPORT_URL="fog://fog.prod.mobilecoinww.com"
        export MC_WALLET_FILE=${MC_WALLET_FILE:-${GIT_BASE}/.mob/mainnet_secret_mnemonic.json}
        export MC_FOG_AUTHORITY_SPKI="MIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEAyr/99fvxi104MLgDgvWPVt01TuTJ+rN4qcNBUbF5i3EMM5zDZlugFHKPYPv7flCh5yDDYyLQHfWkxPQqCBAqlhSrCakvQH3HqDSpbM5FJg7pt0k5w+UQGWvP079iSEO5fMRhjE/lORkvk3/UKr2yIXjZ19iEgP8hlhk9xkI42DSg0iIhk59k3wEYPMGSkVarqlPoKBzx2+11CieXnbCkRvoNwLvdzLceY8QNoLc6h2/nht4bcjDCdB0MKNSKFLVp6XNHkVF66jC7QWTZRA/d4pgI5xa+GmkQ90zDZC2sBc+xfquVIVtk0nEvqSkUDZjv7AcJaq/VdPu4uj773ojrZz094PI4Q6sdbg7mfWrcq3ZQG8t9RDXD+6cgugCTFx2Cq/vJhDAPbQHmCEaMoXv2sRSfOhRjtMP1KmKUw5zXmAZa7s88+e7UXRQC+SS77V8s3hinE/I5Gqa/lzl73smhXx8l4CwGnXzlQ5h1lgEHnYLRFnIenNw/mdMGKlWH5HwHLX3hIujERCPAnGLDt+4MjcUiU0spDH3hC9mjPVA3ltaA3+Mk2lDw0kLrZ4Gv3/Ik9WPlYetOuWteMkR1fz6VOc13+WoTJPz0dVrJsK2bUz+YvdBsoHQBbUpCkmnQ5Ok+yiuWa5vYikEJ24SEr8wUiZ4Oe12KVEcjyDIxp6QoE8kCAwEAAQ=="
        funding_account_id="ea7d2628e31ff7f546b193258e4b99f026521a99c2a5fdb76ac45258405f5b12"
        funding_fs_port="9092"
        port_forward_cmd="kubectl -n dev-wallet-mainnet port-forward svc/full-service 9092:9090"
    ;;
    *)
        echo "ERROR: <network> must be main or test"
        usage
        exit 1
    ;;
esac

# Override the URL and account_id for the funding full-service if you want to use a different one
# Parse to get the host and port
TARGET_FS_URL=${TARGET_FS_URL:-"http://localhost:9090/wallet/v2"}
declare -A TARGET_FS_URL_PARSED
parse_url "${TARGET_FS_URL}" TARGET_FS_URL_PARSED

# Set funding url and parse hostname/port
FUNDING_FS_URL=${FUNDING_FS_URL:-"http://localhost:${funding_fs_port}/wallet/v2"}
declare -A FUNDING_FS_URL_PARSED
parse_url "${FUNDING_FS_URL}" FUNDING_FS_URL_PARSED

FUNDING_ACCOUNT_ID=${FUNDING_ACCOUNT_ID:-${funding_account_id}}
test_success=0

funding_mob()
{
    MC_FULL_SERVICE_PORT=${FUNDING_FS_URL_PARSED[port]} MC_FULL_SERVICE_HOST="${FUNDING_FS_URL_PARSED[proto]}${FUNDING_FS_URL_PARSED[host]}" mob "$@"
}

target_mob()
{
    MC_FULL_SERVICE_PORT=${TARGET_FS_URL_PARSED[port]} MC_FULL_SERVICE_HOST="${TARGET_FS_URL_PARSED[proto]}${TARGET_FS_URL_PARSED[host]}" mob "$@"
}

# Check to see if local full-service is running
if ! curl --connect-timeout 2 -s -f "${TARGET_FS_URL}" >/dev/null
then
    echo "ERROR: Full-Service is not running on ${TARGET_FS_URL}"
    echo "      Please ensure the service is running and accessible."
    exit 1
else
    echo "INFO: Connected successfully to local Full-Service at ${TARGET_FS_URL}"
fi

# Check if the Funding Full-Service is running and accessible
read -r -d '' get_account_status_request <<EOF || true
{
    "method": "get_account_status",
    "params": {
        "account_id": "${FUNDING_ACCOUNT_ID}"
    },
    "jsonrpc": "2.0",
    "id": 1
}
EOF

if ! response=$(curl --connect-timeout 2 -s -f --header 'Content-Type: application/json' "${FUNDING_FS_URL}" -d "${get_account_status_request}")
then
    echo "ERROR: Unable to connect to the Funding Full-Service at ${FUNDING_FS_URL}"
    echo "      with account_id: ${FUNDING_ACCOUNT_ID}"
    echo "      Please ensure the service is running and accessible."
    echo "      If you are running this test locally, you can port-forward to the service:"
    echo "      Connect to the development cluster and run:"
    echo "      ${port_forward_cmd}"
    echo "      OR"
    echo "      Set the FUNDING_ACCOUNT_ID and FUNDING_FS_URL environment variable to the full-service instance you want to use."
    exit 1
fi

if [[ "${response}" =~ "AccountNotFound" ]]
then
    echo "ERROR: Unable to get account status for account_id: ${FUNDING_ACCOUNT_ID}"
    echo "      Please ensure the account_id is correct."
    exit 1
fi

echo "INFO: Connected to the Funding Full-Service at ${FUNDING_FS_URL}"

echo "INFO: check for existing wallet file"
if [[ -e ${MC_WALLET_FILE} ]]
then
    echo "ERROR: existing wallet file found: ${MC_WALLET_FILE}"
    echo "      Please drain funds and remove the file before running this script."
    exit 1
fi
mkdir -p "$(dirname "${MC_WALLET_FILE}")"

echo "INFO: check for poetry"
if ! command -v poetry >/dev/null
then
    echo "INFO: poetry not found, installing..."
    pip install poetry
fi

pushd "${GIT_BASE:?}/python" > /dev/null

echo "INFO: Install mob(lecoin) package"
if ! command -v mob >/dev/null
then
    echo "INFO: mob not found, installing..."
    export PATH="${HOME}/.local/bin:${PATH}"
    pip install .
fi

echo "INFO: install test dependencies"
poetry install

echo "INFO: Create wallet"
if target_mob list | grep test-wallet
then
    echo "ERROR: test-wallet already exists"
    exit 1
fi
target_mob -y create --name test-wallet

echo "INFO: Get wallet address"
local_address=$(target_mob address list test-wallet -i 0)
echo "INFO: ${local_address}"

echo "INFO: fund test wallet with 0.02 MOB"
funding_mob -y send "${funding_account_id}" 0.02 MOB "${local_address}"

echo "INFO: wait for funds to arrive"
local_balance=$(target_mob balance test-wallet | cut -d' ' -f2)
while [[ "${local_balance}" == "0" ]]
do
    local_balance=$(target_mob balance test-wallet | cut -d' ' -f2)
    echo "INFO: balance: ${local_balance}"
    sleep 2
done

echo "INFO: get funding account return address"
funding_account_address=$(funding_mob address list "${funding_account_id}" -i 0)
echo "INFO: ${funding_account_address}"

echo "INFO: export wallet"
target_mob -y export test-wallet --file "${MC_WALLET_FILE}"

echo "INFO: remove wallet so we can test import from file"
target_mob -y remove test-wallet

echo "INFO: run tests"
if MC_FULL_SERVICE_PORT=${TARGET_FS_URL_PARSED[port]} MC_FULL_SERVICE_HOST="${TARGET_FS_URL_PARSED[proto]}${TARGET_FS_URL_PARSED[host]}" poetry run pytest -v
then
    echo "INFO: tests passed"
    test_success=1
else
    # this is a bit silly, but we want to catch failure and finish the script to drain any remaining funds
    # we could trap exits, but there are lots of places where we could exit but not have things set up to
    # drain funds
    echo "ERROR: tests failed"
fi

echo "INFO: look for leftover funds"
balances=$(target_mob balance)
while read -r b
do
    # split out ammount from address and token type
    counter=0
    account=$(echo "${b}" | cut -d' ' -f1)
    amount=$(echo "${b}" | cut -d' ' -f2)
    token=$(echo "${b}" | cut -d' ' -f3)
    if (( $(echo "${amount} 0" | awk '{print ($1 > $2)}') ))
    then
        echo "INFO: found leftover funds: ${b}"
        echo "INFO: drain funds to ${funding_account_address:0:5}...${funding_account_address: -5}"
        echo "mob -y send ${account} all ${token} ..."
        # I think there can be a timing issue when sending funds. If full-service has not yet polled the network, it
        # might think its in sync, but might have transactions pending.  As a work around, we will retry sending.
        while ! target_mob -y send "${account}" all "${token}" "${funding_account_address}"
        do
            sleep 5
            echo "INFO: retrying send"
            counter=$((counter+1))
            if ((counter > 5))
            then
                echo "ERROR: out of retries - failed to send leftover funds back to funding account"
                exit 1
            fi
        done
    fi
done <<< "${balances}"

echo "INFO: remove wallet file"
rm "${MC_WALLET_FILE}"

popd > /dev/null

if ((test_success == 0))
then
    echo "ERROR: tests failed"
    exit 1
fi
