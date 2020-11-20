# wallet-service
A MobileCoin service for wallet implementations.

## Contributing

To add or edit tables:

1. Create a migration with `diesel migration generate <migration_name>`
1. Edit the migrations/<migration_name>/up.sql and down.sql.
1. Run the migration with `diesel migration run`, and test delete with `diesel migration redo`