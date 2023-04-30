# Full Service

Full Service, a [ledger](ledger.md) sync tool and [account](account.md) management system, is a standalone executable
that provides blockchain synchronization and wallet services. It creates encrypted, attested connections to validator
nodes to confirm block headers and build and submit new transactions.&#x20;

To keep the blockchain in sync, Full Service downloads new blocks from cloud storage and checks with its validator nodes
that the block headers are correct. If validation succeeds, the new blocks are added to a local copy of the blockchain
called the Ledger database (LedgerDB). This synchronization occurs periodically ensuring the local copy of the
blockchain is up-to-date for calculating balances and generating transactions.
