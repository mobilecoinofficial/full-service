NAMESPACE=test

WORK_DIR="$HOME/.mobilecoin/${NAMESPACE}"
WALLET_DB_DIR="${WORK_DIR}/wallet-db"
LEDGER_DB_DIR="${WORK_DIR}/ledger-db"
mkdir -p ${WORK_DIR}

(cd ${WORK_DIR} && CONSENSUS_SIGSTRUCT_URI=$(curl -s https://enclave-distribution.${NAMESPACE}.mobilecoin.com/production.json | grep consensus-enclave.css | awk '{print $2}' | tr -d \" | tr -d ,)
curl -O https://enclave-distribution.${NAMESPACE}.mobilecoin.com/${CONSENSUS_SIGSTRUCT_URI})

(cd ${WORK_DIR} && INGEST_SIGSTRUCT_URI=$(curl -s https://enclave-distribution.${NAMESPACE}.mobilecoin.com/production.json | grep ingest-enclave.css | awk '{print $2}' | tr -d \" | tr -d ,)
curl -O https://enclave-distribution.${NAMESPACE}.mobilecoin.com/${INGEST_SIGSTRUCT_URI})

mkdir -p ${WALLET_DB_DIR}
${WORK_DIR}/full-service \
    --wallet-db ${WALLET_DB_DIR}/wallet.db \
    --ledger-db ${LEDGER_DB_DIR} \
    --peer mc://node1.test.mobilecoin.com/ \
    --peer mc://node2.test.mobilecoin.com/ \
    --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node1.test.mobilecoin.com/ \
    --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node2.test.mobilecoin.com/ \
    --fog-ingest-enclave-css ${WORK_DIR}/ingest-enclave.css \
    --import-uri http://localhost:9091
