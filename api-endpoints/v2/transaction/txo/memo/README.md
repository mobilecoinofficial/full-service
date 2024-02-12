# Memo

### Authenticated Sender Memos

Authenticated Sender Memos are a type of memo that is typically included in a txo sent between counterparties by wallets that use MobileCoin's APIs and SDKs. They identify the sender of the txo using a hash of their wallet address and allow the recipient of a txo to further validate that it came from the person that is claiming it came from using the sender's public address.

To validate a sender memo using Full Service, you can call [validate\_sender\_memo](validate-sender-memo.md) with a given txo id and expected sender public address. More information on how Full Service verifies sender memos is [here](../../../../../usage/sender-memos.md#verifiability).

Decoding and validating authenticated sender memos requires using Full Service version v2.9.0 or newer and is only available via the v2 API.



### Destination Memos

Destination Memos are a type of memo that are typically included in change txos sent from a wallet back to itself, using the account's change subaddress index. They document, using the blockchain, the payment made by the wallet, recording an address hash of the payee, the fee paid, and the total outlay.



### Address Hashes

Memos identify senders and recipients using address hashes, 16 byte values represented by full-service as  32 character hex-encoded strings. Use [verify\_address](../../../account/address/verify\_address.md) to get the address hash for a provided B58-encoded public address.

## Example get\_txo memo attribute return contents

{% tabs %}
{% tab title="Authenticated Sender Memo" %}
```json
...
    "memo": {
      "AuthenticatedSender": {
          "sender_address_hash": "12df8df8ddcc6d8830f8cdd1cd00cc51",
          "payment_request_id": null,
          "payment_intent_id": null
      }
    }
...
```
{% endtab %}

{% tab title="Destination Memo" %}
````json
...
    "memo": {
      "Destination": {
        "recipient_address_hash": "8a515f44149956609f75b27d214daed6",
        "num_recipients": "1",
        "fee": "400000000",
        "total_outlay": "1400000000",
        "payment_request_id": null,
        "payment_intent_id": null
      }
    }
```
````
{% endtab %}
{% endtabs %}
