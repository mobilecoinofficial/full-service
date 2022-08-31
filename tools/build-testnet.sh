NAMESPACE=test

CONSENSUS_SIGSTRUCT_URI=$(curl -s https://enclave-distribution.${NAMESPACE}.mobilecoin.com/production.json | grep consensus-enclave.css | awk '{print $2}' | tr -d \" | tr -d ,)
curl -O https://enclave-distribution.${NAMESPACE}.mobilecoin.com/${CONSENSUS_SIGSTRUCT_URI}

INGEST_SIGSTRUCT_URI=$(curl -s https://enclave-distribution.${NAMESPACE}.mobilecoin.com/production.json | grep ingest-enclave.css | awk '{print $2}' | tr -d \" | tr -d ,)
curl -O https://enclave-distribution.${NAMESPACE}.mobilecoin.com/${INGEST_SIGSTRUCT_URI}

SGX_MODE=HW \
IAS_MODE=PROD \
CONSENSUS_ENCLAVE_CSS=$(pwd)/consensus-enclave.css \
INGEST_ENCLAVE_CSS=$(pwd)/ingest-enclave.css \
cargo build