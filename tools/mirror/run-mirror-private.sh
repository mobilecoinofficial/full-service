#!/bin/bash
# Copyright (c) 2022 The MobileCoin Foundation

./bin/wallet-service-mirror-private --mirror-public-uri "wallet-service-mirror://localhost/?ca-bundle=server.crt&tls-hostname=localhost" --wallet-service-uri http://localhost:9090/wallet --mirror-key mirror-private.pem