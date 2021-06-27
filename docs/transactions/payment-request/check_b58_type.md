---
description: 检查一个 B58 编码的类型
---

# 检查 B58 类型

## 参数

| 参数 | 用途 | 说明 |
| :--- | :--- | :--- |
| `b58_code` | 要检查的编码 | 类型为字符串 |

## 示例

{% tabs %}
{% tab title="请求内容" %}
```text
{
  "method": "check_b58_type",
  "params": {
    "b58_code": "3Th9MSyznKV8VWAHAYoF8ZnVVunaTcMjRTnXvtzqeJPfAY8c7uQn71d6McViyzjLaREg7AppT7quDmBRG5E48csVhhzF4TEn1tw9Ekwr2hrq57A8cqR6sqpNC47mF7kHe"
  },
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}

{% tab title="返回" %}
```text
{
  "method": "check_b58_code",
  "result": {
    "type": "PaymentRequest",
    "data": {
      "value": "1000000000000",
      "public_address_b58": "4BfAQbahn9Bs8on7RrWkpargtVUiGNnLrbsmCVFyeqFHHATbwV4CRtjQvhhzpyrkbWBU2HqWK8Fg6boZ235YLEzkGJNFBEVGTKAnCN6vNGV",
      "memo": "testing testing"
    }
  },
  "error": null,
  "jsonrpc": "2.0",
  "id": 1
}
```
{% endtab %}
{% endtabs %}

