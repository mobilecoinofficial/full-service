#!/bin/bash

usage()
{
    echo "Usage:"
    echo "${0} --mnemonic '<24 words>'"
    echo "    --mnemonic - 24 word recover to use - if provided will import the account on startup."
    echo "                 Make sure you quote the whole set of words."
}

while (( "$#" ))
do
    case "${1}" in
        --help | -h)
            usage
            exit 0
            ;;
        --mnemonic)
            mnemonic="${2}"
            shift 2
            ;;
        *)
            echo "${1} unknown option"
            usage
            exit 1
            ;;
    esac
done

if [[ -z "${mnemonic}" ]]
then
    echo "--mnemonic required"
    exit 1
fi

if [[ ! -f "/tmp/mirror_test/mirror-client.pem" ]]
then
    echo "Error: /tmp/mirror_test/mirror-client.pem not found, did you start full-service mirror suite with ./run-full-service-mirror.sh?"
fi

echo "Install node modules"
npm install

echo "Checking that these methods are unsupported."

declare -a MethodList=("assign_address_for_account" "build_and_submit_transaction" "build_gift_code" "build_split_txo_transaction" "build_transaction" "check_b58_type" "check_gift_code_status" "claim_gift_code" "create_account"  "create_receiver_receipts" "export_account_secrets" "get_all_addresses_for_account" "get_all_gift_codes" "get_all_transaction_logs_for_account" "get_all_transaction_logs_ordered_by_block" "get_all_txos_for_account" "get_all_txos_for_address" "get_gift_code" "get_mc_protocol_transaction" "get_mc_protocol_txo" "get_txo" "get_txos_for_account" "import_account" "import_account_from_legacy_root_entropy" "remove_account" "remove_gift_code" "submit_gift_code" "submit_transaction" "update_account_name")

for method in "${MethodList[@]}"; do
    echo "${method}"

    response=$(node mirror-client.js --public-mirror-url http://127.0.0.1:9091 --key-file /tmp/mirror_test/mirror-client.pem --request "{\"method\": \"${method}\", \"params\": {}, \"jsonrpc\": \"2.0\", \"id\": 1}" 2>/dev/null)

    if ! [[ "$response" =~ 'Http error, status: 400: Unsupported request' ]]
    then
        echo "[ FAIL ] $method returned:"
        echo "$response"
        echo "which was not 'Http error, status: 400: Unsupported request'"
        exit 1
    fi
done

echo "Unsupported methods [ PASS ]"

echo "Running mirror-test.sh - supported methods"

if node mirror-test.js --public-mirror-url http://127.0.0.1:9091 \
    --full-service-url http://127.0.0.1:9090 \
    --key-file /tmp/mirror_test/mirror-client.pem \
    --mnemonic "${mnemonic}"
then
    echo "Supported methods [ PASS ]"
else
    echo "Supported methods [ FAIL ]"
    exit 1
fi
