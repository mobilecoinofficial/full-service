# Fog

Two major technical challenges impede privacy-preserving cryptocurrencies from running on smartphones:

1. Identifying received payments. In order to check if they own any new transaction that appears in the [ledger](ledger.md), a user must mathematically test each txo using their private cryptographic keys. It is undesirable from a security standpoint to provision private keys to a remote server to monitor for received transactions, but it is impractical to perform the calculation for every new transaction on a smartphone because of the significant bandwidth and computation required.
2. Constructing new payments. Users need access to the complete [ledger](ledger.md) in order to construct transaction input rings. It is undesirable from a privacy standpoint for smartphone users to selectively download parts of the ledger from a remote server as needed, since this can potentially leak information about transaction ownership or the links between senders and recipients. The complete [ledger](ledger.md) may be many terabytes in size, which makes it impractical to download and store on a smartphone.

MobileCoin Fog is a scalable service infrastructure developed by MobileCoin to enable privacy-preserving cryptocurrencies to be safely managed from a smartphone. MobileCoin Fog solves both of these identified challenges to smartphone deployment.

<figure><img src="../.gitbook/assets/fog.png" alt=""><figcaption></figcaption></figure>

### Identifying Received Payments

MobileCoin has developed an efficient system to help smartphone users locate their received transactions without having to test every transaction using their private cryptographic keys. Each transaction output in the [MobileCoin Ledger](ledger.md) contains a hint field. This field stores the recipient’s [public address](public-address.md), encrypted using a public key provided by MobileCoin Fog. The matching private cryptographic key is stored exclusively inside a [secure enclave](secure-enclave.md). Each new transaction in the public ledger is processed within the secure enclave, and recognized transactions are organized for users who have registered their public address.

Additional data transformations are necessary to safely store persistent data across a scalable service infrastructure and significant care is applied to avoid leaking information through data-access side channels. The user’s private cryptographic keys remain on their smartphone at all times. This stands in contrast to existing trusted query services prevalent in other privacy-preserving cryptocurrencies. When users provision a remote server with private keys, they entrust their privacy to the remote service’s operators.

### Constructing New Payments

MobileCoin Fog allows smartphone users to selectively download ledger data using oblivious data access, meaning that the user can safely retrieve transactions or blocks from the ledger to build new payments without needing to store a complete copy of the blockchain. MobileCoin Fog uses secure enclaves to implement oblivious data access with improved trust and efficiency.
