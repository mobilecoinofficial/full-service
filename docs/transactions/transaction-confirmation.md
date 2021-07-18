---
description: 钱包在构建 TXO 时会生成一个确认编码。
---

# 确认编码

确认编码可以由发送方提供给接收方以证明一个 TXO 确实是由发送方发出的。

## 属性

| 属性 | 类型 | 说明 |
| :--- | :--- | :--- |
| `object` | 字符串，固定为 "confirmation" | 由字符串表示的对象类型。每个类型的 `object` 字段是固定的。 |
| `txo_id` | 字符串 | TXO 的唯一标识符。 |
| `txo_index` | 字符串 | TXO 在账簿上的索引。 |
| `confirmation` | 字符串 | 一个包含确认编码的字符串，可以被验证以证明交易的另一方确实参与了该 TXO 的构造。 |

## 示例

```text
{
  "object": "confirmation",
  "txo_id": "873dfb8c...",
  "txo_index": "1276",
  "confirmation": "984eacd..."
}
```

## 方法

### `get_confirmations`

由钱包构建的 TXO 会包括一个确认编码，可以由发送方分享给接收方，接收方可以据此确认 TXO 和发送方的关联。
当调用 `get_confirmations` 时，系统只会返回对应的确认编码（而不会包括其他的交易细节）。

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `transaction_log_id` | 需要获取确认编码的交易日志 ID。| 该交易日志必须存在在钱包内。|

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "get_confirmations",
  "params": {
    "transaction_log_id": "0db5ac892ed796bb11e52d3842f83c05f4993f2f9d7da5fc9f40c8628c7859a4"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "get_confirmations",
  "result": {
    "confirmations": [
      {
        "object": "confirmation",
        "txo_id": "9e0de29bfee9a391e520a0b9411a91f094a454ebc70122bdc0e36889ab59d466",
        "txo_index": "458865",
        "confirmation": "0a20faca10509c32845041e49e009ddc4e35b61e7982a11aced50493b4b8aaab7a1f"
      }
    ]
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

### `validate_confirmation`

发送方可以向接收方提供交易的确认编码，接收方可以据此验证特定的 TXO ID（在不同钱包间，同一个 TXO 的 TXO ID 不会发生变化。因此，发送方和接收方对于同一个由发送方构建，被接收方接收的 TXO 会有同样的 TXO ID）。

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `account_id` | 用于验证的账户。 | 指定的账户必须存在在钱包中。 |
| `txo_id` | 要与确认编码验证的 TXO ID。 | TXO 必须为已接收状态。 |
| `confirmation` | 要验证的确认编码。 | 确认编码应该由发送方提供。 |


{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "validate_confirmation",
  "params": {
    "account_id": "4b4fd11738c03bf5179781aeb27d725002fb67d8a99992920d3654ac00ee1a2c",
    "txo_id": "bbee8b70e80837fc3e10bde47f63de41768ee036263907325ef9a8d45d851f15",
    "confirmation": "0a2005ba1d9d871c7fb0d5ba7df17391a1e14aad1b4aa2319c997538f8e338a670bb"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "validate_confirmation",
  "result": {
    "verified": true
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1,
}
```
{% endtab %}
{% endtabs %}

