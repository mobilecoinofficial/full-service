#!/bin/bash
# Copyright (c) 2022 The MobileCoin Foundation

NET="$1"

if [ $NET == "main" ]; then
    NAMESPACE="prod"
    PEER_DOMAIN="prod.mobilecoinww.com/"
    TX_SOURCE_URL="https://ledger.mobilecoinww.com"
elif [ $NET == "test" ]; then
    NAMESPACE=$NET
    PEER_DOMAIN="test.mobilecoin.com/"
    TX_SOURCE_URL="https://s3-us-west-1.amazonaws.com/mobilecoin.chain"
elif [ $NET == "alpha" ]; then
    NAMESPACE=$NET
    PEER_DOMAIN="alpha.development.mobilecoin.com/"
    TX_SOURCE_URL="https://s3-eu-central-1.amazonaws.com/mobilecoin.eu.development.chain"
else
    # TODO: add support for local network
    echo "Unknown network"
    echo "Usage: run-fs.sh {main|test|alpha} [--no-build]"
    exit 1

WORK_DIR="$HOME/.mobilecoin/${NET}"
WALLET_DB_DIR="${WORK_DIR}/wallet-db"
LEDGER_DB_DIR="${WORK_DIR}/ledger-db"
INGEST_DOWNLOAD_LOCATION="$WORK_DIR/ingest-enclave.css"
mkdir -p ${WORK_DIR}

## Make sure we have an ingest enclave
# At this time, we cannot download the enclaves for alphanet at all. User must supply
if [ $NET == "alpha" ]; then  
    if ! test -f "$INGEST_DOWNLOAD_LOCATION"; then
        if test -f "$WORK_DIR/libingest-enclave.css"; then
            export INGEST_ENCLAVE_CSS="$WORK_DIR/libingest-enclave.css"
        else
            echo "Please place the ingest enclave for alphanet in $WORK_DIR"
            echo "Ask ops for the consensus and ingest enclaves."
            exit 1
        fi
    else
        export INGEST_ENCLAVE_CSS=$INGEST_DOWNLOAD_LOCATION
    fi
# We always want to download the most recent enclaves for the networks when building the FS binary
elif [ "$2" != "--no-build" ] ; then
    (cd ${WORK_DIR} && INGEST_SIGSTRUCT_URI=$(curl -s https://enclave-distribution.${NAMESPACE}.mobilecoin.com/production.json | grep ingest-enclave.css | awk '{print $2}' | tr -d \" | tr -d ,)
    curl -O https://enclave-distribution.${NAMESPACE}.mobilecoin.com/${INGEST_SIGSTRUCT_URI})
    export INGEST_ENCLAVE_CSS=$INGEST_DOWNLOAD_LOCATION
# We also download if we don't already have INGEST_ENCLAVE_CSS in the environment, use download location
elif [ -z "$INGEST_ENCLAVE_CSS" ]; then
    # If there isn't one in our download location, download
    if ! test -f "$INGEST_DOWNLOAD_LOCATION"; then
        (cd ${WORK_DIR} && INGEST_SIGSTRUCT_URI=$(curl -s https://enclave-distribution.${NAMESPACE}.mobilecoin.com/production.json | grep ingest-enclave.css | awk '{print $2}' | tr -d \" | tr -d ,)
        curl -O https://enclave-distribution.${NAMESPACE}.mobilecoin.com/${INGEST_SIGSTRUCT_URI})
    fi
    export INGEST_ENCLAVE_CSS=$INGEST_DOWNLOAD_LOCATION
# INGEST_ENCLAVE_CSS is set in the environment. Make sure it exists
else
    if ! test -f $INGEST_ENCLAVE_CSS; then
        echo "Missing ingest enclave at $INGEST_ENCLAVE_CSS"
        exit 1
    fi
fi

# Pass "--no-build" if the user just wants to run what they have in  
# WORK_DIR instead of building and copying over a new exectuable
if [ "$2" != "--no-build" ]; then
    echo "Building"
    SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
    $SCRIPT_DIR/build-fs.sh $NET
    cp $SCRIPT_DIR/../target/release/mc-full-service $WORK_DIR
fi 

mkdir -p ${WALLET_DB_DIR}
$WORK_DIR/mc-full-service \
    --wallet-db ${WALLET_DB_DIR}/wallet.db \
    --ledger-db ${LEDGER_DB_DIR} \
    --peer mc://node1.${PEER_DOMAIN} \
    --peer mc://node2.${PEER_DOMAIN} \
    --tx-source-url ${TX_SOURCE_URL}/node1.${PEER_DOMAIN} \
    --tx-source-url ${TX_SOURCE_URL}/node2.${PEER_DOMAIN}  \
    --fog-ingest-enclave-css $INGEST_ENCLAVE_CSS \
    --chain-id $NET


