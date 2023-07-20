---
description: Check the status of a receiver receipt.
---

# Check Receiver Receipt Status

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L78)

| Required Param     | Purpose                                    | Requirements                     |
| ------------------ | ------------------------------------------ | -------------------------------- |
| `address`          | The account's public address.              | Must be a valid account address. |
| `receiver_receipt` | The receipt whose status is being checked. |                                  |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L61)

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "check_receiver_receipt_status",
  "params": {
    "address": "3FDsgJgz4mtGpDFL5cibrKZJgTPcwA8bw4kTDT1j64A6kgPbxgW2QfUS3TbNsjaeBc9wzYyNhcCabtuEjbKhfSc8oLoJLUi9QzomiVBq778",
    "receiver_receipt": {
      "public_key": "0a20167628bd36b6c70aed289cdb3d61d22eb4b40a48f304c484a8f8de781ab54565",
      "confirmation": "0a20d0257c93a691dba8e9aa136e9edb7d6882470e92645ed3e08ea43d8570f0182e",
      "tombstone_block": "1769546",
      "amount": {
        "commitment": "ea71b9404e5dd41be2dfb7c2692fa667551fb8384f583b920ee77440d7cc4c27",
        "masked_value": "15236607299386164772",
        "masked_token_id": "529039b52d15e8ca",
        "version": "V2"
      }
    }
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method": "check_receiver_receipt_status",
  "result": {
    "receipt_transaction_status": "TransactionSuccess",
    "txo": {
      "id": "245669e1ced312bfe5a1a7e99c77918acf7bb5b4e69eb21d8ef74961b8dcc07e",
      "value": "229200000000",
      "token_id": "0",
      "received_block_index": "1769541",
      "spent_block_index": "1769543",
      "account_id": "d43197097fd50aa944dd1b1025d4818668a812f794f4fb4dcf2cab890d3430ee",
      "status": "spent",
      "target_key": "0a20f0eb6416c6da0dfd22c16f4d94de0a7606556b556ed7f5d080baa34a0714f67f",
      "public_key": "0a20167628bd36b6c70aed289cdb3d61d22eb4b40a48f304c484a8f8de781ab54565",
      "e_fog_hint": "0a54c6a878bc8d6da36a47903332336f59b5af7fcfec635c4b914051e762141f5060b52b4e634533904675a289f870faf70dd75f012cafeec0e809fee8d71e831369077d4fd028d7a3f4b9b540f8abe19c62936b0100",
      "subaddress_index": "0",
      "key_image": "0a203401430591cf58da85d5d18a6e606671c20d88019157c2ca02ad6fc5da0dd850",
      "confirmation": "0a20d0257c93a691dba8e9aa136e9edb7d6882470e92645ed3e08ea43d8570f0182e"
    }
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}
