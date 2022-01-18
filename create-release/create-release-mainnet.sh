#!/bin/sh

set -e

RELEASE_NAME="$1"
if [ -z "$RELEASE_NAME" ]; then
    echo "Usage: $0 [release name, e.g. wallet-service-mirror-0.6.0]"
    exit 1
fi

SCRIPT_DIR="$( cd "$( dirname "$0" )" >/dev/null 2>&1 && pwd )"
PROJECT_ROOT="$SCRIPT_DIR/.."
RELEASE_DIR=$SCRIPT_DIR/release/$RELEASE_NAME

export SGX_MODE=HW
export IAS_MODE=PROD
export CONSENSUS_ENCLAVE_CSS=$RELEASE_DIR/consensus-enclave.css
export INGEST_ENCLAVE_CSS=$RELEASE_DIR/ingest-enclave.css

mkdir $RELEASE_DIR

CONSENSUS_PROD_SIGSTRUCT_URI=$(curl -s https://enclave-distribution.prod.mobilecoin.com/production.json | grep consensus-enclave.css | awk '{print $2}' | tr -d \" | tr -d ,)
(cd $RELEASE_DIR && curl -O https://enclave-distribution.prod.mobilecoin.com/${CONSENSUS_PROD_SIGSTRUCT_URI})

INGEST_PROD_SIGSTRUCT_URI=$(curl -s https://enclave-distribution.prod.mobilecoin.com/production.json | grep ingest-enclave.css | awk '{print $2}' | tr -d \" | tr -d ,)
(cd $RELEASE_DIR && curl -O https://enclave-distribution.prod.mobilecoin.com/${INGEST_PROD_SIGSTRUCT_URI})

# Build requires dependencies
cargo build -p mc-full-service --release --manifest-path $PROJECT_ROOT/Cargo.toml

# Create release dir
cp $PROJECT_ROOT/target/release/full-service $RELEASE_DIR/
tar -czvf release/$RELEASE_NAME.tar.gz $RELEASE_DIR/

echo Created $RELEASE_NAME.tar.gz
