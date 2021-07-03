# 检验确认编码

发送方可以向接收方提供交易的确认编码，接收方可以据此验证特定的 TXO ID（在不同钱包间，同一个 TXO 的 TXO ID 不会发生变化。因此，发送方和接收方对于同一个由发送方构建，被接收方接收的 TXO 会有同样的 TXO ID）。

## 参数

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `account_id` | 用于验证的账户。 | 指定的账户必须存在在钱包中。 |
| `txo_id` | 要与确认编码验证的 TXO ID。 | TXO 必须为已接收状态。 |
| `confirmation` | 要验证的确认编码。 | 确认编码应该由发送方提供。 |

## 示例

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

