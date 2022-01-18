#!/bin/sh

set -e

RELEASE_NAME="$1"
if [ -z "$RELEASE_NAME" ]; then
    echo "Usage: $0 [release name, e.g. wallet-service-mirror-0.6.0]"
    exit 1
fi

./create-release-mainnet.sh $RELEASE_NAME
./create-release-testnet.sh $RELEASE_NAME