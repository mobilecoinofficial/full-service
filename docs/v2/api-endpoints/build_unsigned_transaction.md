---
description: >-
  Build a unsigned transaction for use with the offline transaction signer
---

# Build Unsigned Transaction

## [Request](../../../full-service/src/json_rpc/v2/api/request.rs#L67-L74)

| Required Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `account_id` | The account on which to perform this action | Account must exist in the wallet |

| Optional Param | Purpose | Requirements |
| :--- | :--- | :--- |
| `addresses_and_amounts` | An array of public addresses and [Amounts](../../../full-service/src/json_rpc/v2/models/amount.rs) as a tuple | addresses are b58-encoded public addresses |
| `recipient_public_address` | The recipient for this transaction | b58-encoded public address bytes |
| `amount` | The [Amount](../../../full-service/src/json_rpc/v2/models/amount.rs) to send in this transaction |  |
| `input_txo_ids` | Specific TXOs to use as inputs to this transaction | TXO IDs \(obtain from `get_txos_for_account`\) |
| `fee_value` | The fee value to submit with this transaction | If not provided, uses `MINIMUM_FEE` of the first outputs token_id, if available, or defaults to MOB |
| `fee_token_id` | The fee token_id to submit with this transaction | If not provided, uses token_id of first output, if available, or defaults to MOB |
| `tombstone_block` | The block after which this transaction expires | If not provided, uses `cur_height` + 10 |
| `block_version` | string(u64) | The block version to build this transaction for. Defaults to the network block version |
| `max_spendable_value` | The maximum amount for an input TXO selected for this transaction |  |

## [Response](../../../full-service/src/json_rpc/v2/api/response.rs#L52-L56)

## Example

{% tabs %}
{% tab title="Request Body" %}
```
{
  "method": "build_unsigned_transaction",
  "params": {
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
    "recipient_public_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
    "amount": { "value": "42000000000000", "token_id": "0" },
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% hint style="info" %}
Since the `tx_proposal`JSON object is quite large, you may wish to write the result to a file for use in the `submit_transaction` call, such as:

```
{
  "method": "build_unsigned_transaction",
  "params": {
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
    "recipient_public_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
    "value": ["42000000000000", "0"]
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endhint %}

{% tab title="Response" %}
```
{
    "method": "build_unsigned_transaction",
    "result": {
        "account_id": "f85920dd83f69d8850799e28240e3d395f0ad46dec2561b71f4614dd90a3edb5",
        "unsigned_tx": {
            "inputs_and_real_indices_and_subaddress_indices": [
                [
                    {
                        "ring": [
                            {
                                "amount": {
                                    "commitment": {
                                        "point": [
                                            110,
                                            67,
                                            108,
                                            179,
                                            54,
                                            239,
                                            253,
                                            171,
                                            25,
                                            26,
                                            42,
                                            36,
                                            161,
                                            47,
                                            182,
                                            162,
                                            238,
                                            150,
                                            19,
                                            239,
                                            135,
                                            105,
                                            35,
                                            199,
                                            135,
                                            39,
                                            241,
                                            153,
                                            28,
                                            47,
                                            238,
                                            34
                                        ]
                                    },
                                    "masked_value": 13622244978257257768
                                },
                                "target_key": [
                                    244,
                                    70,
                                    136,
                                    223,
                                    79,
                                    40,
                                    46,
                                    133,
                                    26,
                                    58,
                                    228,
                                    199,
                                    51,
                                    186,
                                    193,
                                    201,
                                    129,
                                    44,
                                    96,
                                    80,
                                    91,
                                    41,
                                    103,
                                    74,
                                    9,
                                    240,
                                    9,
                                    160,
                                    33,
                                    251,
                                    13,
                                    86
                                ],
                                "public_key": [
                                    202,
                                    221,
                                    20,
                                    237,
                                    139,
                                    55,
                                    248,
                                    142,
                                    20,
                                    163,
                                    56,
                                    0,
                                    203,
                                    217,
                                    174,
                                    194,
                                    249,
                                    51,
                                    180,
                                    242,
                                    161,
                                    82,
                                    223,
                                    204,
                                    72,
                                    3,
                                    62,
                                    149,
                                    66,
                                    222,
                                    117,
                                    106
                                ],
                                "e_fog_hint": {
                                    "bytes": [
                                        110,
                                        125,
                                        227,
                                        82,
                                        218,
                                        223,
                                        95,
                                        65,
                                        209,
                                        228,
                                        38,
                                        49,
                                        180,
                                        30,
                                        18,
                                        49,
                                        58,
                                        67,
                                        91,
                                        98,
                                        221,
                                        188,
                                        168,
                                        9,
                                        11,
                                        66,
                                        158,
                                        35,
                                        115,
                                        162,
                                        5,
                                        22,
                                        132,
                                        130,
                                        172,
                                        91,
                                        159,
                                        113,
                                        119,
                                        242,
                                        178,
                                        138,
                                        201,
                                        130,
                                        130,
                                        199,
                                        191,
                                        105,
                                        231,
                                        220,
                                        247,
                                        41,
                                        98,
                                        55,
                                        110,
                                        66,
                                        36,
                                        92,
                                        129,
                                        122,
                                        27,
                                        4,
                                        197,
                                        19,
                                        129,
                                        63,
                                        29,
                                        172,
                                        110,
                                        35,
                                        188,
                                        80,
                                        155,
                                        166,
                                        150,
                                        57,
                                        186,
                                        20,
                                        204,
                                        69,
                                        238,
                                        45,
                                        1,
                                        0
                                    ]
                                },
                                "e_memo": null
                            },
            ...
            ],
            "outlays": [
                [
                    "2zXcnQgVixbzzicVCq6mYeiQrFJBcnkDJ2oNYN3dUU5sEJ9DJheKXZseAwkkuCKdhW3KWVjmb4owwHsA3DuhAWaZUQcEjixsjbVvAGUHJ2P",
                    50000000000
                ]
            ],
            "fee": 400000000,
            "tombstone_block_index": 692515
        },
        "fog_resolver": {}
    },
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}
{% endtabs %}
