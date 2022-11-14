# verify that the transaction went through
#   the mob went through
#   the transaction log updatedx
# Ideally all of the endpoints (v2) that actually hit the mobilecoin network
# 
#     get_network_status
#     get_wallet_status
#     build, build_and_submit, build_split_txo .. etc

import sys
import os
sys.path.append(os.path.abspath("../cli"))

from fullservice import FullServiceAPIv2 as v2 
import asyncio
import json
import subprocess

with open('config') as json_file:
    config = json.load(json_file)


def get_account_setup(n = 2):
    # copy over files from the running docker instance and import them
    containerId = config["local_network_containerid"]
    print(containerId)
    return
    for i in range(n):
        filePath = f'/mc-network/sample_data/keys/account_keys_{i}.json'
        containerId = "1234"
        cmd = f"docker cp {containerId}:{filePath} ."
        subprocess.Popen(cmd, shell=True)



async def main():
    get_account_setup()
    return
    fs = v2()

    network_status = fs.get_network_status().result.network_status

    assert(network_status.network_block_height == network_status.local_block_height)
    
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