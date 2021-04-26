---
description: Create an account to receive MOB transactions.
---

# Recieve MOB

Direct your Full Service API calls to localhost:9090/wallet.

## Create an account

1. [Create an account](../accounts/untitled.md#create_account) to receive MOB.
2. [Export account secrets ](../accounts/account-secrets.md#export_account_secrets)to create a mnemonic that will allow you to recover your account. 

{% hint style="warning" %}
Security best practices dictate that your secret mnemonic should only exist in one physical form, usually stamped on a silver plate, and stored securely in a safety deposit box. It is highly discouraged to share your mnemonic, back it up in the cloud, or store it on a device that can be accessed by the internet.
{% endhint %}

## Receive MOB

To receive MOB, you must provide the sender with an account address.

When you create an account, the API response includes a `main_address` that you can share to receive funds. The `main_address` is a subaddress at index 0. You must know which subaddress your MOB was sent to in order to spend it. Limiting the number of subaddresses makes it simpler to keep track of your funds. Using a single address for multiple transactions will anonymize the senders and there will not be a way to verify the amount sent by each sender.

In order to track who is sending what payments, you can create unique subaddresses to share with a particular sender and or a particular transaction.

1. Generate a [subaddress](../accounts/address.md#assign_address_for_account).

