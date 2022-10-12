#!/bin/bash
# Copyright (c) 2022 The MobileCoin Foundation

NET="$1"

if [ "$NET" == "main" ]; then
    NAMESPACE="prod"
    PEER_DOMAIN="prod.mobilecoinww.com/"
    TX_SOURCE_URL="https://ledger.mobilecoinww.com"
    INGEST_SIGSTRUCT_URI=$(curl -s https://enclave-distribution.${NAMESPACE}.mobilecoin.com/production.json | grep ingest-enclave.css | awk '{print $2}' | tr -d \" | tr -d ,)
elif [ "$NET" == "test" ]; then
    NAMESPACE=$NET
    PEER_DOMAIN="test.mobilecoin.com/"
    TX_SOURCE_URL="https://s3-us-west-1.amazonaws.com/mobilecoin.chain"
    INGEST_SIGSTRUCT_URI=$(curl -s https://enclave-distribution.${NAMESPACE}.mobilecoin.com/production.json | grep ingest-enclave.css | awk '{print $2}' | tr -d \" | tr -d ,)
elif [ "$NET" == "alpha" ]; then
    NAMESPACE=$NET
    PEER_DOMAIN="alpha.development.mobilecoin.com/"
    TX_SOURCE_URL="https://s3-eu-central-1.amazonaws.com/mobilecoin.eu.development.chain"
    INGEST_SIGSTRUCT_URI=""
else
    # TODO: add support for local network
    echo "Unknown network"
    echo "Usage: run-fs.sh {main|test|alpha} [--no-build]"
    exit 1
fi

WORK_DIR="$HOME/.mobilecoin/${NET}"
WALLET_DB_DIR="${WORK_DIR}/wallet-db"
LEDGER_DB_DIR="${WORK_DIR}/ledger-db"
INGEST_DOWNLOAD_LOCATION="$WORK_DIR/ingest-enclave.css"
mkdir -p ${WORK_DIR}


if ! test -f "$INGEST_DOWNLOAD_LOCATION" && [ "$INGEST_SIGSTRUCT_URI" != "" ]; then
    (cd ${WORK_DIR} && curl -O https://enclave-distribution.${NAMESPACE}.mobilecoin.com/${INGEST_SIGSTRUCT_URI})
fi

if [ -z "$INGEST_ENCLAVE_CSS" ]; then
    export INGEST_ENCLAVE_CSS=$INGEST_DOWNLOAD_LOCATION
fi

if ! test -f "$INGEST_ENCLAVE_CSS"; then
    echo "Missing ingest enclave at $INGEST_ENCLAVE_CSS"
    exit 1
fi

# Pass "--no-build" if the user just wants to run what they have in  
# WORK_DIR instead of building and copying over a new exectuable
if [ "$2" != "--no-build" ]; then
    echo "Building"
    SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
    $SCRIPT_DIR/build-fs.sh $NET
    cp $SCRIPT_DIR/../target/release/full-service $WORK_DIR
fi 

mkdir -p ${WALLET_DB_DIR}
$WORK_DIR/full-service \
    --wallet-db ${WALLET_DB_DIR}/wallet.db \
    --ledger-db ${LEDGER_DB_DIR} \
    --peer mc://node1.${PEER_DOMAIN} \
    --peer mc://node2.${PEER_DOMAIN} \
    --tx-source-url ${TX_SOURCE_URL}/node1.${PEER_DOMAIN} \
    --tx-source-url ${TX_SOURCE_URL}/node2.${PEER_DOMAIN}  \
    --fog-ingest-enclave-css $INGEST_ENCLAVE_CSS \
    --chain-id $NET
