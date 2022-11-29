# verify that the transaction went through
#   the mob went through
#   the transaction log updatedx
# Ideally all of the endpoints (v2) that actually hit the mobilecoin network
#
#     get_network_status
#     get_wallet_status
#     build, build_and_submit, build_split_txo .. etc

import argparse
import os
import sys
import asyncio
import json

sys.path.append(os.path.abspath("../cli"))

from fullservice import FullServiceAPIv2 as v2
from FSDataObjects import Response, Account 

default_config_path = "./config"
config = []
account_ids = []

fs = v2()


def get_mnemonics(n=2):
    if n > len(config["Account Mnemonics"]):
        raise ValueError("Not enough account available in config")
    return config["Account Mnemonics"][:n]


async def get_account(i):
    global account_ids

    mnemonic = config["Account Mnemonics"][i]["mnemonic"]
    account = await fs.import_account(
        mnemonic, "2"
    )  # this is importing the second mnemonic?

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
    while (await fs.get_wallet_status())['result']['wallet_status']['is_synced_all'] != True:
        await asyncio.sleep(1)  
    print(await does_it_go())


async def does_it_go(amount_pmob: int = 600000000) -> bool:
    network_status = await fs.get_network_status()
    assert "error" not in network_status.keys()
    fee = network_status.get("result").get("network_status").get("fees").get("0") # zero is the fee key for mob

    alice = await get_account(0)
    bob = await get_account(1)
    await fs.get_wallet_status()

    pmob_to_send = amount_pmob
    bob_status_0 = (
        (await fs.get_account_status(bob.id))
        .get("result")
        .get("balance_per_token")
        .get("0")
        .get("unspent")
    )
    alice_status_0 = (
        (await fs.get_account_status(alice.id))
        .get("result")
        .get("balance_per_token")
        .get("0")
        .get("unspent")
    )

    first_transaction = await fs.build_and_submit_transaction(
        alice.id,
        recipient_public_address=bob.main_address,
        amount={"value": str(pmob_to_send), "token_id": str(0)},
    )

    # TODO: replace this with a poll loop that waits a block or two
    await asyncio.sleep(15)
    alice_status_1 = (
        (await fs.get_account_status(alice.id))
        .get("result")
        .get("balance_per_token")
        .get("0")
        .get("unspent")
    )
    bob_status_1 = (
        (await fs.get_account_status(bob.id))
        .get("result")
        .get("balance_per_token")
        .get("0")
        .get("unspent")
    )

    assert alice_status_0 == alice_status_1 + fee + pmob_to_send
    assert bob_status_1 == bob_status_0 + pmob_to_send
    
if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Basic test")
    parser.add_argument("config_path", type=str, default=default_config_path, 
                        required=False)
    args = parser.parse_args()

    print(args.__dict__)
    with open(args.config_path) as json_file:
        config = json.load(json_file)

    asyncio.run(main())
