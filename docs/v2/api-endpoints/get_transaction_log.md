# Get Transaction Log

## [Request](../../../full-service/src/json_rpc/v2/api/request.rs#L40)

| Required Param | Purpose | Requirement |
| :--- | :--- | :--- |
| `transaction_log_id` | The transaction log ID to get. | Transaction log must exist in the wallet. |

## [Response](../../../full-service/src/json_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "get_transaction_log",
  "params": {
    "transaction_log_id": "914e703b5b7bc44b61bb3657b4ee8a184d00e87a728e2fe6754a77a38598a800"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "get_transaction_log",
  "result": {
    "transaction_log":{
      "id": "ab447d73553309ccaf60aedc1eaa67b47f65bee504872e4358682d76df486a87",
      "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
      "value_map": {
        "0": "42000000000000"
      },
      "fee_value": "10000000000",
      "fee_token_id": "0",
      "submitted_block_index": "152950",
      "finalized_block_index": null,
      "status": "pending",
      "input_txos": [
        {
          "txo_id": "eb735cafa6d8b14a69361cc05cb3a5970752d27d1265a1ffdfd22c0171c2b20d",
          "amount": {
            "value": "50000000000",
            "token_id": "0"
          }
        }
      ],
      "payload_txos": [
        {
          "txo_id": "fd39b4e740cb302edf5da89c22c20bea0e4408df40e31c1dbb2ec0055435861c",
          "amount": {
            "value": "30000000000",
            "token_id": "0"
          },
          "recipient_public_address_b58": "vrewh94jfm43m430nmv2084j3k230j3mfm4i3mv39nffrwv43"
        }
      ],
      "change_txos": [
        {
          "txo_id": "bcb45b4fab868324003631b6490a0bf46aaf37078a8d366b490437513c6786e4",
          "amount": {
            "value": "10000000000",
            "token_id": "0"
          },
          "recipient_public_address_b58": "grewmvn3990435vm032492v43mgkvocdajcl2icas"
        }
      ],
      "sent_time": "2021-02-28 01:42:28 UTC",
      "comment": "",
      "failure_code": null,
      "failure_message": null
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

