---
description: 在 Mac 或 Linux 上配置您的 Full Service 环境。
---

# 环境配置

## 二进制

1. 下载[测试网络或主网络的二进制](https://github.com/mobilecoinofficial/full-service/releases)。
2. 打开一个终端窗口并定位到您的下载目录来运行刚下载的 Full Service 程序。

   * 如果您下载的是测试网络版本，请运行：

   ```text
   mkdir -p testnet-dbs
   RUST_LOG=info,mc_connection=info,mc_ledger_sync=info ./full-service \
       --wallet-db ./testnet-dbs/wallet.db \
       --ledger-db ./testnet-dbs/ledger-db/ \
       --peer mc://node1.test.mobilecoin.com/ \
       --peer mc://node2.test.mobilecoin.com/ \
       --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node1.test.mobilecoin.com/ \
       --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node2.test.mobilecoin.com/ \
       --fog-ingest-enclave-css $(pwd)/ingest-enclave.css
   ```

   * 如果您下载的是主网络版本，请运行：

   ```text
     mkdir -p mainnet-dbs
     RUST_LOG=info,mc_connection=info,mc_ledger_sync=info ./full-service \
       --wallet-db ./mainnet-dbs/wallet.db \
       --ledger-db ./mainnet-dbs/ledger-db/ \
       --peer mc://node1.prod.mobilecoinww.com/ \
       --peer mc://node2.prod.mobilecoinww.com/ \
       --tx-source-url https://ledger.mobilecoinww.com/node1.prod.mobilecoinww.com/ \
       --tx-source-url https://ledger.mobilecoinww.com/node2.prod.mobilecoinww.com/ \
       --fog-ingest-enclave-css $(pwd)/ingest-enclave.css
   ```

{% hint style="info" %}
如果您想使用您自己的信任节点组的话，请替换我们的默认 peer 或 tx-source-url 参数。
{% endhint %}

## **HTTP 请求工具**

1. 安装一个您喜欢的 HTTP 请求工具，比如：[Postman](https://www.postman.com/) 用来通过 HTTP 请求和 Full Service 交互。

