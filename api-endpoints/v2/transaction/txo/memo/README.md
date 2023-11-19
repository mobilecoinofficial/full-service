# Memo

### Authenticated Sender Memos

Authenticated Sender Memos are a type of memo that is typically included in a txo sent between counterparties by wallets that use MobileCoin's APIs and SDKs. They identify the sender of the txo using a hash of their wallet address and allow the recipient of a txo to further validate that it came from the person that is claiming it came from using the sender's public address.

To validate a sender memo using Full Service, you can call [validate\_sender\_memo](validate-sender-memo.md) with a given txo id and expected sender public address. More information on how Full Service verifies sender memos is [here](../../../../../usage/sender-memos.md#verifiability).

Decoding and validating authenticated sender memos requires using Full Service version v2.9.0 or newer and is only available via the v2 API.
