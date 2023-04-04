# Sending MobileCoin

The last step in developing your Wallet Interface is to connect to the [build and submit transaction](../../api-endpoints/v2/transaction/transaction/build\_and\_submit\_transaction.md) endpoint. Sending transactions enables the Exchange to send MOB to customers, as well as to transfer MOB between accounts that the Exchange controls.

{% hint style="info" %}
For all transactions, the input txos will be selected automatically unless the input txos are specified during the transaction build step. This is generally not recommended and users should let input txos be automatically selected.
{% endhint %}

Review the initial balance of your account by calling [`get_account_status`](../../api-endpoints/v2/account/account/get\_account\_status.md) with your `account_id` generated from the [Create an Account section](receive-mob.md).

Since you are running a test that doesn't require you to review the [transaction proposal](../../glossary/transaction-proposal.md) before submitting it to the [ledger](../../glossary/ledger.md), call the convenience method [`build_and_submit_transaction`](../../api-endpoints/v2/transaction/transaction/build\_and\_submit\_transaction.md) to send [MOB](../../glossary/mob.md) to a [public address](../../glossary/public-address.md). Keep note of the `transaction_log_id` from the response, as we will need for the next step.

{% tabs %}
{% tab title="Javascript" %}
```javascript
async function buildAndSubmitTransaction() {
 const response = await fetch('http://127.0.0.1:9090/wallet/v2', {
   method: 'POST',
   body: `{
     "method": "build_and_submit_transaction",
     "params": {
         "account_id": "1f32a...",
         "recipient_public_address": "3yNzpvKEpis...",
         "amount": {"value": "1000000000", "token_id": "0"}
     },
     "id": 1,
     "jsonrpc": "2.0"
   }`,
   headers: {
     'Content-Type': 'application/json'
   }
 });
 return await response.json();
}

buildAndSubmitTransaction().then((response) => {
 console.log(response);
});
```
{% endtab %}

{% tab title="Python" %}
```python
from mobilecoin.client import ClientSync, Amount
client = ClientSync()
transaction_log, tx_proposal = client.build_and_submit_transaction(
    account_id='1f32a...',
    amount=Amount.from_display_units(1.0, 'MOB'),
    to_address='3yNzpvKEpis...',
)
```
{% endtab %}
{% endtabs %}

To verify whether the [transaction](../../glossary/transaction.md) was successful, call [`get_transaction_log`](../../api-endpoints/v2/transaction/transaction-log/get\_transaction\_log.md) with the `transaction_log_id` from the previous step. The response will have a field called `status` which should say `succeeded` once the [transaction](../../glossary/transaction.md) clears on the [blockchain](../../glossary/blockchain.md).

{% tabs %}
{% tab title="Javascript" %}
```javascript
async function getTransactionLog() {
 const response = await fetch('http://127.0.0.1:9090/wallet/v2', {
   method: 'POST',
   body: `{
     "method": "get_transaction_log",
     "params": {
         "transaction_log_id": "ewvvf4...1f32a"
     },
     "id": 1,
     "jsonrpc": "2.0"
   }`,
   headers: {
     'Content-Type': 'application/json'
   }
 });
 return await response.json();
}

getTransactionLog().then((response) => {
 console.log(response);
});
```
{% endtab %}

{% tab title="Python" %}
```python
from mobilecoin.client import ClientSync, Amount
client = ClientSync()
transaction_log = client.get_transaction_log(
    transaction_log_id='ewvvf4...'
)
```
{% endtab %}
{% endtabs %}

{% hint style="info" %}
[Transactions](../../glossary/transaction.md) typically get processed by [consensus](../../glossary/consensus-protocol.md) and are added to the [blockchain](../../glossary/blockchain.md) in a few seconds. Once a block has appeared on the [blockchain](../../glossary/blockchain.md), it is finalized and does not require waiting for a certain number of [blocks](../../glossary/block.md) to pass (unlike many other cryptocurrencies).
{% endhint %}

Congratulations, you just sent your first [transaction](../../glossary/transaction.md)!
