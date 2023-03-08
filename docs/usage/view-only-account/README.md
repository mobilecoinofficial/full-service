# View Only Account

Normally, an account in Full Service contains an account key, which is a set of two private keys derived from a mnemonic, the spend private key and the view private key.

With view only accounts, it requires only the view private key which allows a user to view tx outputs that have been received to the account, but doesn't allow a user to generate the data required to spend them. This can be useful if a user wants to take extra precaution with their account and keep the spend portion of it on an offline machine or in a hardware wallet.

Currently, we have support for this through a program called the Transaction Signer which is a stateless program that can be run on an offline machine to generate account information and sign transactions.

Support for hardware wallets such as the Ledger Nano X & S Plus is currently being worked on and will be coming soon!
