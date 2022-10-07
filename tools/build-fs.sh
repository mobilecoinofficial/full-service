#!/bin/bash
# Copyright (c) 2018-2022 The MobileCoin Foundation

# TODO: add a way to specify  'testnet'
# TODO: add a way to specify a different sigstruct directory

# Net can be main/test/local
if [ $# -eq 0 ]; then
NET="test"
else
NET=$1
fi

if [ $NET == "local" ]; then
echo "Local networks are not currently supported with this script"
exit 1
elif [ $NET == "test" ]; then
echo "Building Full Service with testnet settings"
NAMESPACE="test"
SM=SW
IM=DEV
elif [ $NET == "main" ]; then
NAMESPACE="prod"
SM=HW
IM=PROD
else
echo "Network specified is not valid. Try 'test' or 'main'. Default is test"
exit 1
fi

WORK_DIR="$HOME/.mobilecoin/${NET}"
WALLET_DB_DIR="${WORK_DIR}/wallet-db"
LEDGER_DB_DIR="${WORK_DIR}/ledger-db"
mkdir -p ${WORK_DIR}

CONSENSUS_ENCLAVE_CSS="$WORK_DIR/consensus-enclave.css"

(cd ${WORK_DIR} && CONSENSUS_SIGSTRUCT_URI=$(curl -s https://enclave-distribution.${NAMESPACE}.mobilecoin.com/production.json | grep consensus-enclave.css | awk '{print $2}' | tr -d \" | tr -d ,)
curl -O https://enclave-distribution.${NAMESPACE}.mobilecoin.com/${CONSENSUS_SIGSTRUCT_URI})

echo "building full service..."
SGX_MODE=$SM \
IAS_MODE=$IM \
CONSENSUS_ENCLAVE_CSS=$CONSENSUS_ENCLAVE_CSS \
cargo build --release -p mc-full-service

