# Receiving MobileCoin

To receive [MOB](../../glossary/mob.md), you must provide the sender with an account address.

When you created your [account](../../glossary/account.md) in the previous section, the API response included a `main_address` that you can share to receive funds by default. The `main_address` is the [subaddress](../../glossary/subaddress.md) for the [account](../../glossary/account.md) at index 0.

Using the `main_address` generated in the previous section, send a transaction from an account that already has some [MOB](../../glossary/mob.md).

After it has been sent, call [get\_account\_status](../../api-endpoints/v2/account/account/get\_account\_status.md), which should return with a balance of unspent [MOB](../../glossary/mob.md) of the amount that was sent by the sender.

Congratulations, you just received your first transaction!

## Generating a Unique Subaddress

If an Exchange wants to have multiple people paying them, the Exchange will not be able to tell which customers have paid because MobileCoin is private, unlike some other cryptocurrencies. MobileCoin has provided public addresses for subaddresses in order to provide a unique [public address](../../glossary/public-address.md) for each [transaction](../../glossary/transaction.md) with a customer.

All subaddresses up to unsigned int max (18\_446\_744\_073\_709\_551\_615) are automatically associated with an account; however, the Exchange will need to assign a range of subaddresses during or after account creation in order to get the public address for that subaddress or to check the balance to see if the customer has deposited funds at that subaddress.

To generate a new unique subaddress, you can call the [Assign Address For Account endpoint](../../api-endpoints/v2/account/address/assign\_address\_for\_account.md).

{% tabs %}
{% tab title="Javascript" %}
```javascript
async function assignAddressForAccount() {
  const response = await fetch('http://localhost:9090/wallet/v2', {
    method: 'POST',
    body: `{
      "method": "assign_address_for_account",
      "params": {
          "account_id": "1f32a...",
      },
      "jsonrpc": "2.0",
      "id": 1
    }`,
    headers: {
      'Content-Type': 'application/json'
    }
  });
  const json = await response.json();
  return json.result.address;
}

assignAddressForAccount().then((address) => {
  console.log(address.public_address_b58);
});
```

\

{% endtab %}

{% tab title="Python" %}
```python
from mobilecoin.client import ClientSync
client = ClientSync()
address = client.assign_address_for_account(account_id='1f32a...')
print(address['public_address_b58'])
```
{% endtab %}
{% endtabs %}

{% hint style="warning" %}
It is important that you keep track of which subaddress is for which customer depositing to your account
{% endhint %}

Using the public address that we just created, send some MOB to it.

To check the balance of this specific subaddress, you can call the [Get Address Status endpoint](../../api-endpoints/v2/account/address/get\_address\_status.md).

{% tabs %}
{% tab title="Javascript" %}
```javascript
async function getAddressStatus() {
  const response = await fetch('http://localhost:9090/wallet/v2', {
    method: 'POST',
    body: `{
      "method": "get_address_status",
      "params": {
          "address": "1f32a...",
      },
      "jsonrpc": "2.0",
      "id": 1
    }`,
    headers: {
      'Content-Type': 'application/json'
    }
  });
  const json = await response.json();
  return json.result;
}

getAccountStatus().then((accountStatus) => {
  // Show unspent balance for MOB, which is token_id 0.
  // The balance is in pico-MOB, so divide by 1e12 to get whole MOB.
  console.log(`${addressStatus.balance_per_token[0].unspent / 1e12} MOB`);
});
```
{% endtab %}

{% tab title="Python" %}
```python
from mobilecoin.client import (
    ClientSync as Client,
    WalletAPIError
)

account_id = "ae15c..."
client = Client()
public_address = "<public address from previous step>"
status = client.get_address_status(public_address)
balance_per_token = status['balance_per_token']
mob_balance = balance_per_token['0'] # MOB is token 0
print(mob_balance)
```
{% endtab %}
{% endtabs %}
