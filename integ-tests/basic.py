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
import asyncio
import json

sys.path.append(os.path.abspath("../cli")) # lets us import things from the CLI dir 

from fullservice import FullServiceAPIv2 as v2
from dataobjects import Response, Account as FSDataObjects

with open("config") as json_file:
    config = json.load(json_file)

fs = v2()
account_ids = []


def get_mnemonics(n=2):
    if n > len(config["Account Mnemonics"]):
        raise ValueError("Not enough account available in config")
    return config["Account Mnemonics"][:n]


async def get_account(i):
    global account_ids

    mnemonic = config["Account Mnemonics"][i]["mnemonic"]
    account = await fs.import_account(mnemonic, "2")

    if "error" not in account.keys():
        return Account(account["result"]["account"])
    else:
        if len(account_ids) <= i:
            accounts_response = Response(await fs.get_accounts())
            account_ids = accounts_response.account_ids
            return accounts_response.accounts[account_ids[i]]
        else:
            return Response(await fs.get_account_status(account_ids[i])).account


async def main():
    network_status = await fs.get_network_status()

    alice = await get_account(0)
    bob = await get_account(1)
    fs.get_wallet_status()

    pmob_to_send = 485

    first_transaction = await fs.build_and_submit_transaction(
        alice.id,
        recipient_public_address=bob.main_address,
        amount={"value": str(pmob_to_send), "token_id": str(0)},
    )
    
    return 
    
    total_spent = (
        first_transaction.transactionlog.fee_value
        + first_transaction.payload_txos[0].value
    )
    alice2 = fs.get_account_status(alice.account.account_id).result
    assert (
        alice2.balance_per_token["0"].unspent
        == alice.balance_per_token["0"].unspent - total_spent
    )

    log = fs.get_transaction_log(first_transaction.transaction_log.id).result
    assert log.status == "tx_status_succeeded"

    existing_accounts = await fs.get_accounts()
    print(existing_accounts)


if __name__ == "__main__":
    asyncio.run(main())
