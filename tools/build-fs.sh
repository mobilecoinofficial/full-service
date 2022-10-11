#!/bin/bash
# Copyright (c) 2018-2022 The MobileCoin Foundation

# To use this script, run build-fs test or build-fs main
# If no network is specified, or a different network is specified, the env's 
#   version of the following variables are used
#   - SGX_MODE
#   - IAS_MODE
#   - CONSENSUS_ENCLAVE_CSS

# Net can be main/test/local
if [ $# -gt 0 ]; then
    NET=$1
fi

if ([ "$NET" == "test" ] || [ "$NET" == "main" ]); then
    if [ $NET == "test" ]; then
        NAMESPACE="test"
        export SGX_MODE=HW
        export IAS_MODE=PROD
    elif [ $NET == "main" ]; then
        NAMESPACE="prod"
        export SGX_MODE=HW
        export IAS_MODE=PROD
    fi 
    
    WORK_DIR="$HOME/.mobilecoin/${NET}"
    mkdir -p ${WORK_DIR}
    export CONSENSUS_ENCLAVE_CSS="$WORK_DIR/consensus-enclave.css"
    (cd ${WORK_DIR} && CONSENSUS_SIGSTRUCT_URI=$(curl -s https://enclave-distribution.${NAMESPACE}.mobilecoin.com/production.json | grep consensus-enclave.css | awk '{print $2}' | tr -d \" | tr -d ,)
        curl -O https://enclave-distribution.${NAMESPACE}.mobilecoin.com/${CONSENSUS_SIGSTRUCT_URI})
else
    echo "Using current environment's SGX_MODE, IAS_MODE, CONSENSUS_ENCLAVE_CSS"
fi


echo "building full service..."
cargo build --release