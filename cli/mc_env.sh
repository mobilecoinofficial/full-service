set -o allexport

# MobileCoin wallet service.

MC_NETWORK=testnet

MC_DATA=$HOME/.mobilecoin/$MC_NETWORK
RUST_LOG=info
mc_ledger_sync=info

case $MC_NETWORK in

    mainnet)
        MOBILECOIN_CONFIG=$(cat <<EOF
{
    "api-url": "http://127.0.0.1:9090/wallet",
    "executable": "$MC_DATA/full-service",
    "ledger-db": "$MC_DATA/ledger-db",
    "wallet-db": "$MC_DATA/wallet-db/wallet.db",
    "logfile": "$MC_DATA/wallet_server_log.txt",
    "fog-ingest-enclave-css": "",
    "peer": [
        "mc://node1.prod.mobilecoinww.com/",
        "mc://node2.prod.mobilecoinww.com/"
    ],
    "tx-source-url": [
        "https://ledger.mobilecoinww.com/node1.prod.mobilecoinww.com/",
        "https://ledger.mobilecoinww.com/node2.prod.mobilecoinww.com/"
    ]
}
EOF
        )
        ;;

    testnet)
        MOBILECOIN_CONFIG=$(cat <<EOF
{
    "api-url": "http://127.0.0.1:9090/wallet",
    "executable": "$MC_DATA/full-service",
    "ledger-db": "$MC_DATA/ledger-db",
    "wallet-db": "$MC_DATA/wallet-db/wallet.db",
    "logfile": "$MC_DATA/wallet_server_log.txt",
    "fog-ingest-enclave-css": "",
    "peer": [
        "mc://node1.test.mobilecoin.com/",
        "mc://node2.test.mobilecoin.com/"
    ],
    "tx-source-url": [
        "https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node1.test.mobilecoin.com/",
        "https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node2.test.mobilecoin.com/"
    ]
}
EOF
        )
        ;;

    *)
        echo "Invalid MobileCoin network: $MC_NETWORK"
        ;;

esac

set +o allexport
