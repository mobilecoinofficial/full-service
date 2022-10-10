#!/bin/bash
# Copyright (c) 2022 The MobileCoin Foundation
NAMESPACE=prod
NET=main

WORK_DIR="$HOME/.mobilecoin/${NAMESPACE}"
WALLET_DB_DIR="${WORK_DIR}/wallet-db"
LEDGER_DB_DIR="${WORK_DIR}/ledger-db"
mkdir -p ${WORK_DIR}
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

INGEST_ENCLAVE_CSS="$WORK_DIR/ingest-enclave.css"

(cd ${WORK_DIR} && INGEST_SIGSTRUCT_URI=$(curl -s https://enclave-distribution.${NAMESPACE}.mobilecoin.com/production.json | grep ingest-enclave.css | awk '{print $2}' | tr -d \" | tr -d ,)
curl -O https://enclave-distribution.${NAMESPACE}.mobilecoin.com/${INGEST_SIGSTRUCT_URI})

$SCRIPT_DIR/build-fs.sh $NET

# Default is to run whatver binary is sitting in the directory under mobilecoin named $NAMESPACE
# However, the build script by default drops the binary in the release folder.
if [ $# -eq 0 ]; then
FS_DIR=$WORK_DIR
else
FS_DIR="$SCRIPT_DIR/../target/release"
fi

mkdir -p ${WALLET_DB_DIR}
${FS_DIR}/full-service \
    --wallet-db ${WALLET_DB_DIR}/wallet.db \
    --ledger-db ${LEDGER_DB_DIR} \
    --peer mc://node1.$NAMESPACE.mobilecoinww.com/ \
    --peer mc://node2.$NAMESPACE.mobilecoinww.com/ \
    --tx-source-url https://ledger.mobilecoinww.com/node1.$NAMESPACE.mobilecoinww.com/ \
    --tx-source-url https://ledger.mobilecoinww.com/node2.$NAMESPACE.mobilecoinww.com/ \
    --fog-ingest-enclave-css $INGEST_ENCLAVE_CSS \
    --chain-id $NET
