---
description: 'Generate txo confirmations from the sender and validate them on the receiver'
---

# TXO Confirmations

## Account Prep

Make sure you have 2 accounts ready, with one of them funded with some MOB. If you still need to set up some accounts, refer to the [`Run Full Service`](./environment-setup.md) section.

It is only important that 1 account has MOB on it, which we will call `account_a` from here on out. `account_a` will be the sending account, and `account_b` will be the receiving account.

## Send a Transaction

Send a transaction from `account_a` to `account_b` by calling [`build_and_submit_transaction`](../transactions/transaction/build_and_submit_transaction.md) and providing the main public address of `account_b`

This will return a response that will include the `transaction_log_id`, which we will need for the next step

## Get The TXO Confirmations

1. Using the `transaction_log_id` obtained from the previous step, call [`get_confirmations`](../transactions/transaction-confirmation/get_confirmations.md)

2. This will return a response that has an array of confirmations. In most cases, there will only be 1, but if you created a multi-output transaction there will be more.

3. For the TXO that you wish to confirm, this is where you would send the recipient the txo_id and confirmation pairing for each txo you wish to have them validate.

## Validate The TXO Confirmation

1. For each of the txo_id and confirmation pairings generated in the previous step, call [`validate_confirmation`](../transactions/transaction-confirmation/validate_confirmation.md) using the `account_id` of `account_b`, which is the receiving account.

If all is successful, you should have gotten a response with a result of `"validated": true`