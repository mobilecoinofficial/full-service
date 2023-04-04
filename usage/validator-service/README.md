# Validator Service

A service that is capable of syncing the ledger from the consensus network, relaying transactions to it and proxying fog report resolution.

The Ledger Validator Node exposes a GRPC service that provides access to its local ledger, transaction relaying and fog report request relaying.&#x20;

Using the `--validator` command line argument for `full-service`, this allows running `full-service` on a machine that is not allowed to make outside connections to the internet\
but can connect to a host running the LVN.
