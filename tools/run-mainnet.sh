#!/bin/bash
# Copyright (c) 2022 The MobileCoin Foundation
NAMESPACE=prod
NET=main

WORK_DIR="$HOME/.mobilecoin/${NET}"
WALLET_DB_DIR="${WORK_DIR}/wallet-db"
LEDGER_DB_DIR="${WORK_DIR}/ledger-db"
INGEST_DOWNLOAD_LOCATION="$WORK_DIR/ingest-enclave.css"

# Pass "--no-build" if the user just wants to run what they have in  
# .mobilecoin/main instead of building and copying over a new exectuable
if [ $# -eq 0 ] || [ $1 != "--no-build" ]; then
    echo "Building"
    mkdir -p ${WORK_DIR}
    SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

    (cd ${WORK_DIR} && INGEST_SIGSTRUCT_URI=$(curl -s https://enclave-distribution.${NAMESPACE}.mobilecoin.com/production.json | grep ingest-enclave.css | awk '{print $2}' | tr -d \" | tr -d ,)
    curl -O https://enclave-distribution.${NAMESPACE}.mobilecoin.com/${INGEST_SIGSTRUCT_URI})
    INGEST_ENCLAVE_CSS=$INGEST_DOWNLOAD_LOCATION

    $SCRIPT_DIR/build-fs.sh $NET
    cp $SCRIPT_DIR/../target/release/full-service $WORK_DIR
fi


if [ -z "$INGEST_ENCLAVE_CSS" ] ; then
    if ! test -f "$INGEST_DOWNLOAD_LOCATION"; then
        (cd ${WORK_DIR} && INGEST_SIGSTRUCT_URI=$(curl -s https://enclave-distribution.${NAMESPACE}.mobilecoin.com/production.json | grep ingest-enclave.css | awk '{print $2}' | tr -d \" | tr -d ,)
        curl -O https://enclave-distribution.${NAMESPACE}.mobilecoin.com/${INGEST_SIGSTRUCT_URI})
        INGEST_ENCLAVE_CSS=$INGEST_DOWNLOAD_LOCATION
    else
        echo "If running with --no-build, please export a path for the fog-ingest-enclave-css as INGEST_ENCLAVE_CSS"
        exit 1
    fi
fi


mkdir -p ${WALLET_DB_DIR}
$WORK_DIR/full-service \
    --wallet-db ${WALLET_DB_DIR}/wallet.db \
    --ledger-db ${LEDGER_DB_DIR} \
    --peer mc://node1.$NAMESPACE.mobilecoinww.com/ \
    --peer mc://node2.$NAMESPACE.mobilecoinww.com/ \
    --tx-source-url https://ledger.mobilecoinww.com/node1.$NAMESPACE.mobilecoinww.com/ \
    --tx-source-url https://ledger.mobilecoinww.com/node2.$NAMESPACE.mobilecoinww.com/ \
    --fog-ingest-enclave-css $INGEST_ENCLAVE_CSS \
    --chain-id $NET
