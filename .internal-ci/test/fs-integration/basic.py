# verify that the transaction went through
#   the mob went through
#   the transaction log updatedx
# Ideally all of the endpoints (v2) that actually hit the mobilecoin network
#
#     get_network_status
#     get_wallet_status
#     build, build_and_submit, build_split_txo .. etc

import argparse
import asyncio
import json
import subprocess
import sys
from typing import Optional
from fullservice import FullServiceAPIv2 as v2
from dataclasses import dataclass
from dataclasses_json import dataclass_json

repo_root_dir = subprocess.check_output("git rev-parse --show-toplevel", shell=True).decode("utf8").strip()
sys.path.append("{}/python-library".format(repo_root_dir))

default_config_path = "./test_config.json"

fs = v2()

# TODO: These dataclasses should be upstreamed to the FS library itself.. it should always returns a Response object  
@dataclass_json
@dataclass
class AccountStatus:
    result: str
    balance_per_token: str
    unspent: str

@dataclass_json
@dataclass
class Response:
    method: str
    result: dict
    jsonrpc: str
    id: int
    error: Optional[dict] = None

#fix me later
async def test_cleanup():
    for id in account_ids:
        await fs.remove_account(id)

def get_mnemonics(n=2):
    if n > len(config["Account Mnemonics"]):
        raise ValueError("Not enough account available in config")
    return config["Account Mnemonics"][:n]

#to be moved to a standard test library. this doesn't belong in the entrypoint script that gets ran by the CI IMO
async def get_account(index, already_imported=False):
    mnemonic = config["Account Mnemonics"][index]["mnemonic"]
    import_resp = Response(await fs.import_account(
        mnemonic, "2"  # This parameter indicates that we are using the 2nd key derivations method (mnemonics)
    )   )

    accounts_response = Response(await fs.get_accounts())
    account_ids = accounts_response.result.get("account_ids")

    if not already_imported:
        assert not import_resp.error,  "Failed to import account"
        return import_resp.result.get("account")

    if len(account_ids) <= index:
        accounts_response = Response(await fs.get_accounts())
        account_ids = accounts_response.result.get("account_ids")
        return accounts_response.accounts[account_ids[index]]
    else:
        return Response(await fs.get_account_status(account_ids[index])).account


async def main():
    while (await fs.get_wallet_status())['result']['wallet_status']['is_synced_all'] != True:
        await asyncio.sleep(1)  
    #print(await does_it_go()) #commented for testing


async def does_it_go(amount_pmob: int = 600000000) -> bool:
    network_status = await fs.get_network_status()
    assert "error" not in network_status.keys(),  "Failed to get network status"
    fee = int(network_status.get("result")
                            .get("network_status")
                            .get("fees")
                            .get("0")  # zero is the fee key for mob
    ) # this get chain is fine for now and shouldn't break

    """Test Setup """

    alice = await get_account(0, True)
    bob = await get_account(1, True)
    await test_cleanup()

    alice = await get_account(0)
    bob = await get_account(1)

    pmob_to_send = amount_pmob
    alice_balance_0 = int(
        (await fs.get_account_status(alice.id))
        .get("result")
        .get("balance_per_token")
        .get("0")
        .get("unspent") 
    )

    assert alice_balance_0 >= pmob_to_send + fee, "Insufficient funds in first account."

    bob_balance_0 = int(
        (await fs.get_account_status(bob.id))
        .get("result")
        .get("balance_per_token")
        .get("0")
        .get("unspent")
    )

    """ Test action """

    first_transaction = await fs.build_and_submit_transaction(
        alice.id,
        recipient_public_address=bob.main_address,
        amount={"value": str(pmob_to_send), "token_id": str(0)},
    )


    """ Check Results """

    # TODO: replace this with a poll loop that waits a block or two
    await asyncio.sleep(5)
    alice_balance_1 = int(
        (await fs.get_account_status(alice.id))
        .get("result")
        .get("balance_per_token")
        .get("0")
        .get("unspent")
    )

    bob_balance_1 = int(
        (await fs.get_account_status(bob.id))
        .get("result")
        .get("balance_per_token")
        .get("0")
        .get("unspent") 
    )

    assert alice_balance_0 == alice_balance_1 + fee + pmob_to_send, "Alice doesn't end with the expected amount"
    assert bob_balance_1 == bob_balance_0 + pmob_to_send, "Bob doesn't end with the expected amount"

    # TODO: stick this in a finally block
    await test_cleanup()

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Basic test")
    parser.add_argument("config_path", nargs='?', type=str, default=default_config_path)
    args = parser.parse_args()

    with open(args.config_path) as json_file:
        config = json.load(json_file)

    asyncio.run(main())
