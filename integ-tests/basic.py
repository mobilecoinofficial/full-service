# verify that the transaction went through
#   the mob went through
#   the transaction log updatedx
# Ideally all of the endpoints (v2) that actually hit the mobilecoin network
# 
#     get_network_status
#     get_wallet_status
#     build, build_and_submit, build_split_txo .. etc

import os
import sys

sys.path.append(os.path.abspath("../cli"))

import asyncio
import json

from fullservice import FullServiceAPIv2 as v2

with open('config') as json_file:
    config = json.load(json_file)

def get_mnemonics(n = 2):
    if n > len(config['Account Mnemonics']):
        raise ValueError("Not enough account available in config")

    return config['Account Mnemonics'][:n]

async def main():
    fs = v2()
    network_status = await fs.get_network_status()

    m = get_mnemonics()
    alice = await fs.import_account(m[0]['mnemonic'], "2")
    assert('error' not in alice.keys())
    alice = alice['result']['account']


    return    
    alice = fs.import_account().result.account
    alice = fs.get_account_status(alice.account_id).result
    bob = fs.import_account().result.account
    bob = fs.get_account_status(bob.account_id).result


    first_transaction = fs.build_and_submit_transaction(
        alice.account.account_id,
        recipient_public_address = bob.account.main_address,
        amount = { "value" : str(1), "token_id": str(0)}
    ).result


    total_spent = first_transaction.transactionlog.fee_value + first_transaction.payload_txos[0].value
    alice2 = fs.get_account_status(alice.account.account_id).result
    assert(alice2.balance_per_token["0"].unspent == 
           alice.balance_per_token["0"].unspent - total_spent)

    
    log = fs.get_transaction_log(first_transaction.transaction_log.id).result
    assert(log.status == "tx_status_succeeded")
    

    existing_accounts = await fs.get_accounts()
    print(existing_accounts)

if __name__ == '__main__':
    asyncio.run(main())
