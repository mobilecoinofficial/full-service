#!/bin/bash
# Copyright (c) 2022 The MobileCoin Foundation

NET="$1"
WORK_DIR="$HOME/.mobilecoin/${NET}"

$WORK_DIR/wallet-service-mirror-private --mirror-public-uri "wallet-service-mirror://localhost/?ca-bundle=$WORK_DIR/server.crt&tls-hostname=localhost" --wallet-service-uri http://localhost:9090/wallet --mirror-key $WORK_DIR/mirror-private.pem