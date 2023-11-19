# Sender Memos

Authenticated Sender Memos are a type of memo that is typically included in a txo sent between counterparties by wallets that use MobileCoin APIs and SDKs. They identify the sender of the txo using a hash of their wallet address and allow the recipient of a txo to further validate that it came from the person that is claiming it came from using the sender's public address.

To validate a sender memo using Full Service, you can call [validate\_sender\_memo](../api-endpoints/v2/transaction/txo/memo/validate-sender-memo.md) with a given txo id and expected sender public address.

Decoding and validating authenticated sender memos requires using Full Service version v2.9.0 or newer and is only available via the v2 API.

### Verifiability

The `authenticated sender memo` includes a hash of the sender's public address, and an HMAC value. These are used to inform the recipient of who the sender is in a verifiable way.

The HMAC is a signed hash of a text concatenation that includes the txo's unique identifier, the memo type bytes, and the memo contents (minus the HMAC field).  The key used to sign the HMAC is formed from a key exchange between the sender's `spend key`, and the recipient's `view key`. This key can be constructed (and verified) by either the sender, using the recipient's _public_ `view key` and their own _private_ `spend key`, OR the recipient, using their own _private_ `view key` and the sender's _public_ `spend key`.

When full-service validates the sender memo for the recipient, it verifies that sender knew the _private_ `spend key` associated with the hashed public address. full-service can locally construct the HMAC using the txo; the recipient's _private_ `view key`, which it knows; and, the sender's _public_ `spend key`, which it can extract from the provided sender public address. To complete the validation, full-service hashes the provided sender public address and compares the self-computed HMAC and hash with those in the txo's `authenticated sender memo.`
