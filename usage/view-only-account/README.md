# View Only Account

Normally, an account in Full Service contains an account key, which is a set of two private keys derived from a
mnemonic, the spend private key and the view private key.

For view-only accounts, the private spend key is stored offline, and transactions are signed on the offline machine with
the transaction signer executable. To view transactions on the online machine, we upload the view private key. This view
key does not allow a user to spend funds. The reason someone might want to do this is to keep extra security on spending
funds, in case the online machine is compromised.

The transaction signer executable is a stateless program that is run offline. It is used to generate account keys,
identify which transactions belong to the account, and sign transactions to spend funds.

Support for hardware wallets such as the Ledger Nano X, and the Ledger Nano S Plus is currently in development in
the [ledger-mob](https://github.com/mobilecoinofficial/ledger-mob) repo, and will be arriving soon!
