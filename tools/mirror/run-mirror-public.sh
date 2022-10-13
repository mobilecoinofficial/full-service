#!/bin/bash
# Copyright (c) 2022 The MobileCoin Foundation

./bin/wallet-service-mirror-public --client-listen-uri http://0.0.0.0:9091/ --mirror-listen-uri "wallet-service-mirror://0.0.0.0/?tls-chain=server.crt&tls-key=server.key" --allow-self-signed-tls