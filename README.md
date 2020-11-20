# wallet-service
A MobileCoin service for wallet implementations.

## Build and Run

    ```
    cargo build --release
    ./target/release/wallet-service
    ```

## API

### Create Account

    Create a new account in the wallet. 
    
    ```
    curl -s localhost:9090/wallet -d '{"method": "create_account", "params": {"name": "Alice"}}' -X POST -H 'Con
    tent-type: application/json' | jq
    
    {
      "method": "create_account",
      "result": {
        "public_address": "7v96ox3enVb9rb7vPsXvUM27uGhSmJNeXpXSyRrV5rUGUSfnSeDE8nnHBeYVe2Debcz4L7JMzwBsTZEJ3Z2kdKDwjcHT4pr7cnntCi29vfr",
        "entropy": "60d1cbeeaf2b23758178213240b07d28930d55151635fca5a1ddbeed50b107f4",
        "account_id": "f4b21ea17e31adc9e15bfd380edfefe90be68adc1214696395cb97133331a7ef"
      }
    }
    ```

## Contributing

To add or edit tables:

1. Create a migration with `diesel migration generate <migration_name>`
1. Edit the migrations/<migration_name>/up.sql and down.sql.
1. Run the migration with `diesel migration run`, and test delete with `diesel migration redo`