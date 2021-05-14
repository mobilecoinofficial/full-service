---
description: '请将您的 Full Service API 请求指向：localhost:9090/wallet'
---

# 运行 Full Service

## 打开账户

### 创建一个新的账户

1. 使用 [`create_account`](../accounts/account/create_account.md) 方法来创建一个新的账户。
2. 为了避免遗忘您的账户，请通过 [`export_account_secrets`](../accounts/account-secrets/export_account_secrets.md) 方法来创建一个用来找回账户的助记词。

{% hint style="warning" %}
导出助记词是找回账户的唯一途径。
{% endhint %}

### 导入一个既存账户

1. 如果您已经有一个账户，您可以通过 [`import_account`](../accounts/account/import_account.md) 方法来导入它。
   * 您必须提供助记词和账户名作为身份认证。
   * 您可以通过指定从特定的区块下标开始扫描账簿以加速导入的过程。如果没有指定的话，Full Service 将会扫描整个账簿，因此耗时将会随着账簿变大而变长。

## MOB 交易

您必须向发送方提供您的账户地址才能接收 MOB。

当您通过 API 创建账户时，返回值中会包括一个 `main_address` 字段，您可以公开这个地址用来收取 MOB。`main_address` 是下标为 0 的子地址。您必须知道 MOB 被发送到的子地址才能够使用它。通过限制您的子地址的数量可以帮助您记录资金流入的历史。如果只使用一个地址进行多笔交易，那么交易的发送方将会不可能被区分，因此也无法得知每个发送方发送的具体金额。

如果想要记录交易的发送方，您可以为特定的发送方或特定的交易创建一个独特的子地址来进行交易。

### 收取 MOB

1. 子地址可以通过 [`assign_address_for_account`](../accounts/address/assign_address_for_account.md) 方法生成。
2. 调用 [`get_wallet_status`](../wallet/wallet-status/get_wallet_status.md) 方法，可以通过 `total_unspent_pmob` 字段查看您收到的金额。

### 发送 MOB

1. 通过调用 [`get_balance_for_account`](../accounts/balance/get_balance_for_account.md) 方法并提供您的 `account_id` 作为参数来查看您的初始余额。
2. 因为测试交易，所以您并不需要去查看 tx\_proposal，因此可以直接通过 [`build_and_submit_transaction`](../transactions/transaction/build_and_submit_transaction.md) 方法来向一个公开的地址发送 MOB。
3. 通过调用 [`get_balance_for_account`](../accounts/balance/get_balance_for_account.md) 方法来比较您的账户在交易前后的余额，并您的交易是否已经成功：
   * 如果您是发送给自己的账户，那么从您的初始余额中减去未花 MOB 交易手续费。
   * 如果您是发送给他人的账户，那么从您的初始余额中减去未花 MOB 交易手续费和您发送的金额。

