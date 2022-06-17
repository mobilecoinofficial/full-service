#!/usr/bin/python3
# Copyright (c) 2018-2022 The MobileCoin Foundation

import subprocess
import os

PROJECT_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
MOBILECOIN_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'mobilecoin'))
CARGO_FLAGS = '--release'
LEDGER_BASE = os.path.join(PROJECT_DIR, 'target', "sample_data", "ledger")

print('building full service...')
cmd=' '.join([
  'RUST_LOG=debug'
  'MC_SEED=a4aa76e4a5ca70c8447dd544a63f180b5a6fe0aff96495802506354c10f2886e',
  'SGX_MODE=SW',
  'IAS_MODE=DEV',
  f'CONSENSUS_ENCLAVE_CSS={MOBILECOIN_DIR}/consensus-enclave.css',
  'cargo build --release -p mc-full-service'
])
# build = subprocess.run(cmd, shell=True, stdout=subprocess.PIPE, universal_newlines=True)
# build
os.system(cmd)