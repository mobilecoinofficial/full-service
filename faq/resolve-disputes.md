---
description: >-
  Resolve disputes by generating txo confirmations from the sender and
  validating them on the receiver
---

# Resolving Disputes

In some cases when sending a transaction, the recipient will report not having received the transaction. Please use the following steps to triage and resolve disputes.

## Verify Transaction Success

First, verify whether the transaction was a success by examining the `transaction_log` for the transaction using the [`get_transaction_log`](../api-endpoints/v2/transaction/transaction-log/get\_transaction\_log.md) endpoint, which will provide an example result as below:

```json
{
  "id":"01cf3c1a5ac2a6b884ef81c1bdd2191a3860d59158118b08f1f8f61ec3e09567",
  "account_id":"d43197097fd50aa944dd1b1025d4818668a812f794f4fb4dcf2cab890d3430ee",
  "input_txos":[
    {
      "txo_id":"fa737a8e65e480fc7f75dbc17e6875b75cf4b14f3cde02b49b8cd8921fdf7dbb",
      "amount":{
        "value":"5999600000000",
        "token_id":"0"
      }
    }
  ],
  "output_txos":[
    {
      "txo_id":"454c511ddab33edccc4b686b67d1f9a6c4eb101c28386e0f4e21c994ea35aa2f",
      "public_key":"728e73bd8675562ab44dea5c2b0edd4bfdf037a73d4afd42267442337c60f73b",
      "amount":{
        "value":"1234600000000",
        "token_id":"0"
      },
      "recipient_public_address_b58":"41mZTnbwQ3E73ZrPQnYPdU7G6Dj3ZrYaBkrcAYPNgm61P7gBvzUke94HQB8ztPaAu1y1NCFyUAoRyYsCMixeKpUvMK64QYC1NDd7YneACJk"
    }
  ],
  "change_txos":[
    {
      "txo_id":"34f8a29a2fdd2446694bf175e533c6bf0cd4ecac9d52cd793ef06fc011661b89",
      "public_key":"3c0225fab2d6df245887b7acebf22c238ffafa54842ab2663ac27833975a2212",
      "amount":{
        "value":"4764600000000",
        "token_id":"0"
      },
      "recipient_public_address_b58":"f7YRA3PsMRNtGaPnxXqGE8Z6eaaCyeAvZtvpkze86aWxcF7a4Kcz1t7p827GHRqM93iWHvqqrp2poG1QxX4xVidAXNuBGzwpCsEoAouq5h"
    }
  ],
  "value_map":{
    "0":"1234600000000"
  },
  "fee_amount":{
    "value":"400000000",
    "token_id":"0"
  },
  "submitted_block_index":"1352852",
  "tombstone_block_index":"1352860",
  "finalized_block_index":"1352852",
  "status":"succeeded",
  "sent_time":null,
  "comment":""
}
```

### Confirm Status and Block Index

For a successful transaction, the `status` is `succeeded`, and the `finalized_block_index` is populated.

### Confirm Recipient Address

The `output_txos` for the transaction contain details about the txo itself, including the `value` and `token_id`. The `recipient_public_address_b58` specifies the address to which the amount was sent.

{% hint style="info" %}
The precision of the value of the txo depends on which token it is. To see more info about the precision of each token and what token are supported, check out our [Supported Tokens](../usage/supported-token-metadata.md) page.
{% endhint %}

### Confirm with the Block Explorer

You can use the `txo_id` from the `output_txos` to get more information about the specific txo over which there may be a dispute, with the [`get_txo`](../api-endpoints/v2/transaction/txo/get\_txo.md) endpoint.

```json
{
  "method":"get_txo",
  "result":{
    "txo":{
      "id": "c50c2d1fbeae481e8bf68e90692f537a9d9fca62177d411d37dbb88e19a8f4d6",
      "value": "9600000000",
      "token_id": "0",
      "received_block_index": null,
      "spent_block_index": null,
      "account_id": null,
      "status": "orphaned",
      "target_key": "0a207ecb238d6b97dc1f83745f7a713eec75d9bce821889498b1e475144c89059c66",
      "public_key": "0a208e279a2326d585a9c8b24c0b3c9d356e63f1c4276b22bb97f25b45bb58ec810d",
      "e_fog_hint": "0a5497815c16a0ed7cc0b40eb3f0d3b4f6e9c2d14455495f971c2eff3e53889ee50e48a0ceede2c502499a4a1f181a8be0e9c0347d1a21923510c86ac102a400b5b5dd4ce8b145ab754f6541d610957857f983cb0100",
      "subaddress_index": null,
      "key_image": null,
      "confirmation": "0a209d298c11da7d6f3798c7ddef69aea407170cc8f917c5cbfb4e8651513995db31"
    }
  },
  "jsonrpc":"2.0",
  "id":1
}
```

You can use the `public_key` to confirm with the block explorer. The MobileCoin Foundation hosts a public block explorer at [https://block-explorer.mobilecoin.foundation/](https://block-explorer.mobilecoin.foundation/).

For the example above, we can go to the block index indicated in the `transaction_log`: `318163`, here: [https://block-explorer.mobilecoin.foundation/blocks/318163](https://block-explorer.mobilecoin.foundation/block/318163), and scroll down to the Transaction Outputs section, where we see the public key `8e279a2326d585a9c8b24c0b3c9d356e63f1c4276b22bb97f25b45bb58ec810d` (with the prefix `0a20` removed)

## Provide Confirmation Receipt to the Receiver

After you have confirmed that the `transaction_log` indicates that you sent the transaction to the correct recipient, and you confirmed that the transaction outputs are in the blockchain, the next step of dispute resolution involves providing a cryptographic proof that you created the transaction, called a _confirmation_. You can follow the steps below to create a confirmation:

### Sender Provides the TXO Confirmations

1. Using the `transaction_log_id` obtained from the previous step, call [`get_confirmations`](../api-endpoints/v2/transaction/transaction-confirmation/get\_confirmations.md)
2. This will return a response that has an array of confirmations.&#x20;

{% hint style="info" %}
In most cases, there will only be 1 confirmation, but if you created a multi-output transaction there will be more (1 for each output txo).
{% endhint %}

```json
{
    "method": "get_confirmations",
    "result": {
        "confirmations": [
            {
                "txo_id": "e5b05ee9db56b0a123bea472284ea24e153d99ef9698ca5dfb4dee56e3320295",
                "txo_index": "4832213",
                "confirmation": "0a2013640c9c2547dc02b9e5d5abe4adb15cc1532c4651a253b661ad7b15e0ebad62"
            }
        ]
    },
    "jsonrpc": "2.0",
    "id": 1
}
```

3. For the TXO that you wish to confirm, this is where you would send the recipient the `txo_id` `txo_index` and `confirmation` for each txo you wish to have them validate.

### Receiver Validates the TXO Confirmation

1. For each of the confirmations generated in the previous step, call [`validate_confirmation`](../api-endpoints/v2/transaction/transaction-confirmation/validate\_confirmation.md) using the `account_id` of receiving account.

If all is successful, you should have gotten a response with a result of `"validated": true`
