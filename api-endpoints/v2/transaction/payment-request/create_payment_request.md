---
description: Create a payment request b58 code to give to someone else
---

# Create Payment Request

## [Request](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/request.rs#L86)

| Required Param | Purpose                                                                                                                                               | Requirements                      |
| -------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------- |
| `account_id`   | The account on which to perform this action.                                                                                                          | Account must exist in the wallet. |
| `amount`       | The [Amount](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/models/amount.rs) to send in this transaction |                                   |

| Optional Param     | Purpose                                                          | Requirements |
| ------------------ | ---------------------------------------------------------------- | ------------ |
| `subaddress_index` | The subaddress index on the account to generate the request with | `i64`        |
| `memo`             | Memo for the payment request                                     |              |

## [Response](https://github.com/mobilecoinofficial/full-service/blob/main/full-service/src/json\_rpc/v2/api/response.rs#L41)

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "create_payment_request",
  "params": {
    "account_id": "60ef9401f98fc278cd8a1ef51f466111244c9d4b97e8f8886a86bd840238dcaa",
    "amount": { "value": "528000000000", "token_id": "0" },
    "subaddress_index": 1,
    "memo": "Payment for dinner with family"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method": "create_payment_request",
  "result": {
    "payment_request_b58": "37Vz37cUTxcrB8PVMLFedtdz1dS9xcV4TGYsCfwjW1jEuGTMtQVRgptZ7xi571gaRhUxk3j9HLjoEGMD7VMQWGHX7PWJD5qcAYPA1SB96WdikREV2azdqoyJvdrgyCT5wt9e8KtmjkoVcHeB1whY6NjD9yEevJVv5GU"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}
