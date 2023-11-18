# Sender Memos

Sender Memos are a type of memo that may be included in a txo. It allows the recipient of a txo to validate that it came from the person that is claiming it came from. For more information about Sender Memos (and other types of memos), see [this document](https://github.com/mobilecoinfoundation/mcips/blob/main/text/0004-recoverable-transaction-history.md#sender-memo).

To validate a sender memo using Full Service, you can call [validate\_sender\_memo](../api-endpoints/v2/transaction/txo/memo/validate-sender-memo.md) with a given txo id and expected sender public address.
