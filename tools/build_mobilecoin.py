#!/usr/bin/python3
# Copyright (c) 2018-2022 The MobileCoin Foundation

import os
import subprocess
PROJECT_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'mobilecoin'))
CARGO_FLAGS = '--release'
LEDGER_BASE = os.path.join(PROJECT_DIR, 'target', "sample_data", "ledger")

# Default log configuration
if 'MC_LOG' not in os.environ:
    os.environ['MC_LOG'] = 'debug,rustls=warn,hyper=warn,tokio_reactor=warn,mio=warn,want=warn,rusoto_core=error,h2=error,reqwest=error,rocket=error,<unknown>=error'

if __name__ == '__main__':
    print('Building binaries...')
    enclave_pem = os.path.join(PROJECT_DIR, 'Enclave_private.pem')
    if not os.path.exists(enclave_pem):
        subprocess.run(
            f'openssl genrsa -out {enclave_pem} -3 3072',
            shell=True,
            check=True,
        )

    subprocess.run(
        f'cd {PROJECT_DIR} && CONSENSUS_ENCLAVE_PRIVKEY="{enclave_pem}" cargo build -p mc-consensus-service -p mc-ledger-distribution -p mc-admin-http-gateway -p mc-util-grpc-admin-tool -p mc-mint-auditor -p mc-crypto-x509-test-vectors -p mc-consensus-mint-client -p mc-util-seeded-ed25519-key-gen {CARGO_FLAGS}',
        shell=True,
        check=True,
    )

