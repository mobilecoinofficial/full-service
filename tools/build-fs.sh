#!/bin/bash
# Copyright (c) 2018-2022 The MobileCoin Foundation

# To use this script, run build-fs test or build-fs main
# If no network is specified, or a different network is specified, the env's 
#   version of the following variables are used
#   - SGX_MODE
#   - IAS_MODE
#   - CONSENSUS_ENCLAVE_CSS

# Net can be main/test/local
NET="$1"

if [ "$NET" == "test" ]; then
    NAMESPACE="test"
    export SGX_MODE=HW
    export IAS_MODE=PROD
    CONSENSUS_SIGSTRUCT_URI=$(curl -s https://enclave-distribution.${NAMESPACE}.mobilecoin.com/production.json | grep consensus-enclave.css | awk '{print $2}' | tr -d \" | tr -d ,)
elif [ "$NET" == "main" ]; then
    NAMESPACE="prod"
    export SGX_MODE=HW
    export IAS_MODE=PROD
    CONSENSUS_SIGSTRUCT_URI=$(curl -s https://enclave-distribution.${NAMESPACE}.mobilecoin.com/production.json | grep consensus-enclave.css | awk '{print $2}' | tr -d \" | tr -d ,)
elif [ "$NET" == "alpha" ]; then
    NAMESPACE="alpha"
    export SGX_MODE=HW
    export IAS_MODE=DEV
    CONSENSUS_SIGSTRUCT_URI=""
else
    echo "Using current environment's SGX_MODE, IAS_MODE, CONSENSUS_ENCLAVE_CSS"
    CONSENSUS_SIGSTRUCT_URI=""
    if [ "$NET" == "" ]; then
        NET="default"
    fi
fi 
    
WORK_DIR="$HOME/.mobilecoin/${NET}"
CONSENSUS_DOWNLOAD_LOCATION="$WORK_DIR/consensus-enclave.css"
mkdir -p ${WORK_DIR}

if ! test -f "$CONSENSUS_DOWNLOAD_LOCATION" && [ "$CONSENSUS_SIGSTRUCT_URI" != "" ]; then
    (cd ${WORK_DIR} && curl -O https://enclave-distribution.${NAMESPACE}.mobilecoin.com/${CONSENSUS_SIGSTRUCT_URI})
fi

if [ -z "$CONSENSUS_ENCLAVE_CSS" ]; then
    export CONSENSUS_ENCLAVE_CSS=$CONSENSUS_DOWNLOAD_LOCATION
fi

if ! test -f "$CONSENSUS_ENCLAVE_CSS"; then
    echo "Missing consensus enclave at $CONSENSUS_ENCLAVE_CSS"
    exit 1
fi

echo "building full service..."
cargo build --release
