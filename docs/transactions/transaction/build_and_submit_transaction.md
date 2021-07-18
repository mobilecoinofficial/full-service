---
description: 一个同时进行构建交易和提交交易的便利方法。
---

# 构建并提交交易

## 参数

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `account_id` | 用来构建并提交交易的账户。 | 指定的账户必须存在在钱包中。 |
| `recipient_public_address` | 当笔交易的收取方。| 字节形式的经 Base 58 编码的公共地址 |
| `value_pmob` | 当笔交易要发送的 MOB 数额。 | 单位是 pmob  |

| 可选参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `addresses_and_values` | 一个由公共地址和 MOB 数额二元组构成的数组。| 地址和数额的形式和上述两个字段一致 |
| `input_txo_ids` | 指定当笔交易内要发送的 MOB （TXO） ID。 | TXO ID \(通过 `get_all_txos_for_account` 获取\) |
| `fee` | 当笔交易的手续费 | 默认值 `MINIMUM_FEE` = .01 MOB |
| `tombstone_block` | 当笔交易的失效期（区块高度） | 默认值为当前区块高度 + 50 |
| `max_spendable_value` | 会被选入当笔交易的 TXO 的最大金额。|  |
| `comment` | 在交易日志中标记当笔交易的备注。 |  |

## 示例

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "build_and_submit_transaction",
  "params": {
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
    "recipient_public_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
    "value_pmob": "42000000000000"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "build_and_submit_transaction",
  "result": {
    "transaction_log": {
      "object": "transaction_log",
      "transaction_log_id": "937f102052500525ff0f54aa4f7d94234bd824260bfd7ba40d0561166dda7780",
      "direction": "tx_direction_sent",
      "is_sent_recovered": null,
      "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
      "recipient_address_id": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
      "assigned_address_id": null,
      "value_pmob": "42000000000000",
      "fee_pmob": "10000000000",
      "submitted_block_index": "152948",
      "finalized_block_index": null,
      "status": "tx_status_pending",
      "input_txo_ids": [
        "8432bb4e25f1bde68e4759b27ec72d290252cb99943f2f38a9035dba230895b7"
      ],
      "output_txo_ids": [
        "135c3861be4034fccb8d0b329f86124cb6e2404cd4debf52a3c3a10cb4a7bdfb"
      ],
      "change_txo_ids": [
        "44c03ddbccb33e5c37365d7b263568a49e6f608e5e818db604541cc09389b762"
      ],
      "sent_time": "2021-02-28 01:27:52 UTC",
      "comment": "",
      "failure_code": null,
      "failure_message": null,
      "offset_count": 2199
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

{% hint style="warning" %}
`如果账户没有完全同步，您可能会看到如下错误信息：`

```text
{
  "error": "Connection(Operation { error: TransactionValidation(ContainsSpentKeyImage), total_delay: 0ns, tries: 1 })"
}
```

调用 `check_balance` 来查看该账户的余额并注意 `synced_blocked` 字段。如果该值小于 `local_block_index` 字段，那么您所持有的 MOB 状态可能并没有完全同步。
{% endhint %}

