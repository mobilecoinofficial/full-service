# Get Transaction Logs For Account

## [Request](../../../full-service/src/json_rpc/v2/api/request.rs#L40)

| Optional Param | Purpose | Requirement |
| :--- | :--- | :--- |
| `account_id` | The account id to scan for transaction logs | Account must exist in the database |
| `min_block_index` | The minimum block index to find transaction logs from | |
| `max_block_index` | The maximum block index to find transaction logs from | |
| `offset` | The pagination offset. Results start at the offset index. | |
| `limit` | Limit for the number of results. | |

## [Response](../../../full-service/src/json_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```text
{
  "method": "get_transaction_logs",
  "params": {
    "account_id": "b59b3d0efd6840ace19cdc258f035cc87e6a63b6c24498763c478c417c1f44ca",
    "offset": 2,
    "limit": 1
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```text
{
  "method": "get_transaction_logs",
  "result": {
    "transaction_log_ids": [
      "ff1c85e7a488c2821110597ba75db30d913bb1595de549f83c6e8c56b06d70d1",
      "58729797de0929eed37acb45225d3631235933b709c00015f46bfc002d5754fc",
      "243494a0030bcbac40e87670b9288834047ef0727bcc6630a2fe2799439879ab"
    ],
    "transaction_log_map": {
      "ff1c85e7a488c2821110597ba75db30d913bb1595de549f83c6e8c56b06d70d1": {
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
            }
            "recipient_public_address_b58": "vrewh94jfm43m430nmv2084j3k230j3mfm4i3mv39nffrwv43"
          }
        ],
        "change_txos": [
          {
            "txo_id": "bcb45b4fab868324003631b6490a0bf46aaf37078a8d366b490437513c6786e4",
            "amount": {
              "value": "10000000000",
              "token_id": "0"
            }
            "recipient_public_address_b58": "grewmvn3990435vm032492v43mgkvocdajcl2icas"
          }
        ],
        "sent_time": "2021-02-28 01:42:28 UTC",
        "comment": "",
        "failure_code": null,
        "failure_message": null
      },
      "58729797de0929eed37acb45225d3631235933b709c00015f46bfc002d5754fc": {
        "id": "58729797de0929eed37acb45225d3631235933b709c00015f46bfc002d5754fc",
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
      },
      "243494a0030bcbac40e87670b9288834047ef0727bcc6630a2fe2799439879ab": {
        "id": "243494a0030bcbac40e87670b9288834047ef0727bcc6630a2fe2799439879ab",
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
    }
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}
