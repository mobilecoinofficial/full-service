NAMESPACE=alpha

WORK_DIR="$HOME/.mobilecoin/${NAMESPACE}"
WALLET_DB_DIR="${WORK_DIR}/wallet-db"
LEDGER_DB_DIR="${WORK_DIR}/ledger-db"
mkdir -p ${WORK_DIR}

mkdir -p ${WALLET_DB_DIR}
${WORK_DIR}/full-service-113 \
    --wallet-db ${WALLET_DB_DIR}/wallet.db \
    --ledger-db ${LEDGER_DB_DIR} \
    --peer mc://node1.alpha.development.mobilecoin.com/ \
    --peer mc://node2.alpha.development.mobilecoin.com/ \
    --tx-source-url https://s3-eu-central-1.amazonaws.com/mobilecoin.eu.development.chain/node1.alpha.development.mobilecoin.com/ \
    --tx-source-url https://s3-eu-central-1.amazonaws.com/mobilecoin.eu.development.chain/node2.alpha.development.mobilecoin.com/ \
    --fog-ingest-enclave-css ${WORK_DIR}/ingest-enclave.css
