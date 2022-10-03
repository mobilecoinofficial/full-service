#!/bin/bash
# Copyright (c) 2018-2022 The MobileCoin Foundation

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
PROJECT_DIR=$SCRIPT_DIR/../mobilecoin

echo "building full service..."
export CONSENSUS_ENCLAVE_CSS="$PROJECT_DIR/target/release/consensus-enclave.css"
cargo build --release -p mc-full-service
