# Create an Account

### Create a New Account

Call [`create_account`](../../api-endpoints/v2/account/account/create\_account.md) to create a new account

{% tabs %}
{% tab title="Javascript" %}
```javascript
 const response = await fetch('http://localhost:9090/wallet/v2', {
   method: 'POST',
   body: `{
     "method": "create_account",
     "params": {
         "name": "Joshua Goldbard",
     },
     "jsonrpc": "2.0",
     "id": 1
 }`,
   headers: {
     'Content-Type': 'application/json'
   }
 });
 const myJson = await response.json();
 account = myJson.result.account
```
{% endtab %}

{% tab title="Python" %}
```python
from mobilecoin.client import ClientSync
client = ClientSync()
account = client.create_account()
print(account)
```
{% endtab %}

{% tab title="Curl" %}
```bash
curl -s localhost:9090/wallet/v2 \
    -X POST \
    -H 'Content-type: application/json' \
    -d '{
        "method": "create_account",
        "params": {},
        "jsonrpc": "2.0",
        "id": 1
    }' \
    | jq
```
{% endtab %}
{% endtabs %}

This method will return a number of parameters including the accounts **id**, the **main\_address** to receive assets, and the **first\_block\_index**.

To protect yourself from ever losing your the associated funds in the account, run [`export_account_secrets`](../../api-endpoints/v2/account/account-secrets/export\_account\_secrets.md) using the **account\_id** from the previous step to show the _secret_ _mnemonic_ that can be used to import the account again

{% tabs %}
{% tab title="Javacript" %}
```javascript
 const response = await fetch('http://localhost:9090/wallet/v2', {
   method: 'POST',
   body: `{
     "method": "export_account_secrets",
     "params": {
         "account_id": "b504409093f....2d87a35d02c",
     },
     "jsonrpc": "2.0",
     "id": 1
 }`,
   headers: {
     'Content-Type': 'application/json'
   }
 });
 const myJson = await response.json();
 account_secrets = myJson.result.account_secrets;
```
{% endtab %}
{% endtabs %}

{% hint style="danger" %}
An account's secret mnemonic is extremely sensitive and anyone with this information will be able to see and spend from your account! Please use best practices for securing and storing this information as **without it you will lose all access to your account and funds.**
{% endhint %}

### Import an Existing Account

If you already have an account, you can access it with the [`import_account`](../../api-endpoints/v2/account/account/import\_account.md) method.

{% tabs %}
{% tab title="Javascript" %}
```javascript
 const response = await fetch('http://localhost:9090/wallet/v2', {
   method: 'POST',
   body: `{
     "method": "import_account",
     "params": {
        "mnemonic": "sheriff odor square mistake huge skate mouse shoot purity weapon proof stuff correct concert blanket neck own shift clay mistake air viable stick group",
        "key_derivation_version": "2",
        "first_block_index": "1200000"
     },
     "jsonrpc": "2.0",
     "id": 1
   }`,
   headers: {
     'Content-Type': 'application/json'
   }
 });
 const myJson = await response.json();
 account = myJson.result.account;
```
{% endtab %}
{% endtabs %}

{% hint style="info" %}
If you have an account that was created before the use of mnemonics, please use the [Import From Legacy Root Entropy](../../api-endpoints/v2/account/account/import\_account\_from\_legacy\_root\_entropy.md) api call.
{% endhint %}

{% hint style="info" %}
To identify your account, you must provide the method with your **secret mnemonic** and an account **name** to be used, however the **name** parameter is optional
{% endhint %}

{% hint style="info" %}
To speed up the import process, you can provide the method with the **first block index** that you'd like to scan from the ledger. If you don’t include the first block index, it will default to scanning the entire ledger, which will take longer as the ledger size increases.
{% endhint %}
