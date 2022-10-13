#!/bin/bash
# Copyright (c) 2022 The MobileCoin Foundation

NET="$1"
WORK_DIR="$HOME/.mobilecoin/${NET}"

$WORK_DIR/wallet-service-mirror-public --client-listen-uri http://0.0.0.0:9091/ --mirror-listen-uri "wallet-service-mirror://0.0.0.0/?tls-chain=$WORK_DIR/server.crt&tls-key=$WORK_DIR/server.key" --allow-self-signed-tls