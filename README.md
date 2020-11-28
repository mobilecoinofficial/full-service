# wallet-service
A MobileCoin service for wallet implementations.

## Build and Run

    1. Get the appropriate published enclave measurement, and save to `$(pwd)/consensus-enclave.css`

        ```
        NAMESPACE=test
        SIGNED_ENCLAVE_URI=$(curl -s https://enclave-distribution.${NAMESPACE}.mobilecoin.com/production.json | grep consensus-enclave.css | awk '{print $2}' | tr -d \" | tr -d ,)
        curl -O https://enclave-distribution.${NAMESPACE}.mobilecoin.com/${SIGNED_ENCLAVE_URI}
        ```

    1. Build

        ```
        SGX_MODE=HW IAS_MODE=PROD CONSENSUS_ENCLAVE_CSS=$(pwd)/consensus-enclave.css cargo build --release
        ```

    1. Run

        ```
        ./target/release/wallet-service \
            --wallet-db /tmp/wallet-db/wallet.db \
            --ledger-db /tmp/ledger-db/ \
            --peer mc://node1.test.mobilecoin.com/ \
            --peer mc://node2.test.mobilecoin.com/ \
            --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node1.test.mobilecoin.com/ \
            --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node2.test.mobilecoin.com/
        ```

## API

### Accounts

#### Create Account

    Create a new account in the wallet. 
    
    ```
    curl -s localhost:9090/wallet -d '{"method": "create_account", "params": {"name": "Alice"}}' -X POST -H 'Content-type: application/json'  | jq
    
    {
      "method": "create_account",
      "result": {
        "public_address": "8JtpPPh9mV2PTLrrDz4f2j4PtUpNWnrRg8HKpnuwkZbj5j8bGqtNMNLC9E3zjzcw456215yMjkCVYK4FPZTX4gijYHiuDT31biNHrHmQmsU",
        "entropy": "c593274dc6f6eb94242e34ae5f0ab16bc3085d45d49d9e18b8a8c6f057e6b56b",
        "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
      }
    }
    ```

#### Import Account

    Import an existing account from the secret entropy.

    ```curl -s localhost:9090/wallet -d '{"method": "import_account", "params": {"entropy": "c593274dc6f6eb94242e34ae5f0ab16bc3085d45d49d9e18b8a8c6f057e6b56b", "name": "Alice"}}' -X POST -H 'Content-type: application/json'  | jq

       {
         "method": "import_account",
         "result": {
           "public_address": "8JtpPPh9mV2PTLrrDz4f2j4PtUpNWnrRg8HKpnuwkZbj5j8bGqtNMNLC9E3zjzcw456215yMjkCVYK4FPZTX4gijYHiuDT31biNHrHmQmsU",
           "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
         }
       }
    ```

#### List Accounts

    ```
    curl -s localhost:9090/wallet -d '{"method": "list_accounts"}' -X POST -H 'Content-type: application/json'  | jq
    
    {
      "method": "list_accounts",
      "result": {
        "accounts": [
          "c7155cb1660f6dfe778dd52f6381ad3a25f35bd9f502ec337b17478f51abaade",
          "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
        ]
      }
    }
    ```

#### Get Account

    ```
    curl -s localhost:9090/wallet -d '{"method": "get_account", "params": {"account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"}}' -X POST -H 'Content-type: application/json'  | jq
    
    {
      "method": "get_account",
      "result": {
        "name": "Alice",
        "balance": "0"
      }
    }
    ```

#### Update Account Name

    ```
    curl -s localhost:9090/wallet -d '{"method": "update_account_name", "params": {"acount_id": "2b2d5cce6e24f4a396402fcf5f036890f9c06660f5d29f8420b8c89ef9074cd6", "name": "Eve"}}' -X POST -H 'Content-type: application/json'  | jq
    {
      "method": "update_account_name",
      "result": {
        "success": true
      }
    }
    ```

#### Delete Account

    ```
    curl -s localhost:9090/wallet -d '{"method": "delete_account", "params": {"account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"}}' -X POST -H 'Content-type: application/json'  | jq
    
    {
      "method": "delete_account",
      "result": {
        "success": true
      }
    }
    ```

### TXOs

#### List TXOs for a given account

    ```
    curl -s localhost:9090/wallet -d '{"method": "list_txos", "params": {"account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"}}' -X POST -H 'Content-type: application/json'  | jq

    {
      "method": "list_txos",
      "result": {
        "txos": [
          {
            "txo_id": "000d688cfe28ab128a7514148f700dc6872e97c1498753fdef4fdd8b90601cd1",
            "value": "97582349900010990",
            "txo_type": "received",
            "txo_status": "spent"
          },
          {
            "txo_id": "00a92e639f2601e9af3ba796c62087cc1c6b9d1bc7c4921df4b136d134ff4027",
            "value": "1",
            "txo_type": "received",
            "txo_status": "spent"
          },
          {
            "txo_id": "00ae2c1a638296dbfe0514019e4efa03b0c714c45b391f1d2180a2c50a38ffad",
            "value": "1",
            "txo_type": "received",
            "txo_status": "spent"
          },
          {
            "txo_id": "00d4f35588ed694edaf58762be9edf3a3cb6941f2a9de3ee779f7c91c3a064a0",
            "value": "97584329900010990",
            "txo_type": "received",
            "txo_status": "spent"
          },
        ]
      }
    }
    ```

#### Get Balance for a given account

    ```
    curl -s localhost:9090/wallet -d '{"method": "get_balance", "params": {"account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"}}' -X POST -H 'Content-type: application/json'  | jq

    {
      "method": "get_balance",
      "result": {
        "balance": "97580449900010991"
      }
    }
    ```

## Contributing

### Database Schema

To add or edit tables:

1. Create a migration with `diesel migration generate <migration_name>`
1. Edit the migrations/<migration_name>/up.sql and down.sql.
1. Run the migration with `diesel migration run`, and test delete with `diesel migration redo`

### Running Tests

    FIXME: I'm not sure why we need to provide these vars for cargo test...

    ```
    SGX_MODE=HW IAS_MODE=DEV CONSENSUS_ENCLAVE_CSS=$(pwd)/consensus-enclave.css cargo test
    ```