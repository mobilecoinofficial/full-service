---
description: >-
  Create a view-only account by importing the private key from an existing
  account. Note: a single wallet cannot have both the regular and view-only versions of an account.
---

# Import

## Parameters

| Required Param | Purpose                                                                            | Requirements |
| -------------- | ---------------------------------------------------------------------------------- | ------------ |
| `package`      | The view only account import package generated from the offline transaction signer |              |

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
    "method": "import_view_only_account",
    "params": {
          "account": {
              "object": "view_only_account",
              "account_id": "f85920dd83f69d8850799e28240e3d395f0ad46dec2561b71f4614dd90a3edb5",
              "name": "ts-test-2",
              "first_block_index": "0",
              "next_block_index": "0",
              "main_subaddress_index": "0",
              "change_subaddress_index": "1",
              "next_subaddress_index": "2"
          },
          "secrets": {
              "object": "view_only_account_secrets",
              "view_private_key": "0a20f6fdc6e12fc60c39fe10be71a0ad7b2e6aaae98d56d59c6a71e3f4043b628b0c",
              "account_id": "f85920dd83f69d8850799e28240e3d395f0ad46dec2561b71f4614dd90a3edb5"
          },
          "subaddresses": [
              {
                  "object": "view_only_subaddress",
                  "public_address": "6MZ9Na9yC6upiE5BSe9gNsBX5zjwjuCASGNGmfvU8cCyWqo6xePySAU84zaMmSi3Zjrt2AKKXPcsy4J1CDmXmoZtFFo9QQ7cgpbUg8opX1y",
                  "account_id": "f85920dd83f69d8850799e28240e3d395f0ad46dec2561b71f4614dd90a3edb5",
                  "comment": "Main",
                  "subaddress_index": "0",
                  "public_spend_key": "0a208650eb2c525a41bcd88ce47dcf8f657bbe0882461ccace1afbc856e22e929348"
              },
              {
                  "object": "view_only_subaddress",
                  "public_address": "6QYeh2h5WegDWGqFYgennj8vjaFzaFTmMZo5M84Ntcsnc69mLSdxrReKditxwLedBSktXznUrC4L3Q57vwiFzfHTXB2EgWQU8LHMB4UjBrj",
                  "account_id": "f85920dd83f69d8850799e28240e3d395f0ad46dec2561b71f4614dd90a3edb5",
                  "comment": "Change",
                  "subaddress_index": "1",
                  "public_spend_key": "0a20ee1adb69b3d6cb3173f712790ff1fe89a1312d678c82cb8e8c940ef9c9e8ed4c"
              }
          ]
    },
    "jsonrpc": "2.0",
    "api_version": "2",
    "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```
{
  "method": "import_view_only_account",
  "result": {
    "account": {
      "object": "view_only_account_account",
      "account_id": "6ed6b79004032fcfcfa65fa7a307dd004b8ec4ed77660d36d44b67452f62b470",
      "name": "Coins for cats",
      "first_block_index": "3500",
      "next_block_index": "4000",
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}
