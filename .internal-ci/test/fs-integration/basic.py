import argparse
import asyncio
import json
import subprocess
import sys

repo_root_dir = subprocess.check_output("git rev-parse --show-toplevel", shell=True).decode("utf8").strip()
sys.path.append("{}/python-library".format(repo_root_dir))

from fullservice import FullServiceAPIv2 as v2
from FSDataObjects import Response, Account 

sleepy_time = 5
default_config_path = "./test_config.json"
config = []
account_ids = []

fs = v2()


async def wait_for_account_to_sync(id):
    account_status = await fs.get_account_status(id)
    while (account_status.get("result").get("account").get("next_block_index")
           != account_status.get("result").get("local_block_height")):
        await asyncio.sleep(sleepy_time)
        account_status = await fs.get_account_status(id)


async def test_cleanup():
    global account_ids
    for id in account_ids:
        await wait_for_account_to_sync(id)
        await fs.remove_account(id)
    accounts = await fs.get_accounts()
    for id in account_ids:
        assert id not in accounts.get('result').get('account_ids'),"Failed to clear out accounts"
    account_ids = []


# If this test fails before reaching the last cleanup step, we have leftover
# artifacts in the FS instance. We clean up those residual artifacts here.
# Note: Using a testing framework like pytest would allow us to bundle this in
# a test fixture
async def preclean_this_test():
    await get_account(0, "alice", True)
    await get_account(1, "bob", True)
    await test_cleanup()


def get_mnemonics(n=2):
    if n > len(config["Account Mnemonics"]):
        raise ValueError("Not enough account available in config")
    return config["Account Mnemonics"][:n]


async def get_account(i, name="", okay_if_already_imported=False):
    global account_ids

    mnemonic = config["Account Mnemonics"][i]["mnemonic"]
    account = await fs.import_account(
        mnemonic,
        "2",  # This parameter indicates that we are using the 2nd key derivations method (mnemonics)
        name=name
    )  

    if not okay_if_already_imported:
        assert "error" not in account.keys(),  "Failed to import account"

    if "error" not in account.keys():
        result = Account(account["result"]["account"])
        account_ids.append(result.id)
        return result
    else:
        if len(account_ids) <= i:
            accounts_response = Response(await fs.get_accounts())
            account_ids = accounts_response.account_ids
            return accounts_response.accounts[account_ids[i]]
        else:
            result = Response(await fs.get_account_status(account_ids[i])).account
            account_ids.append(result.id)
            return result


async def main():
    while (await fs.get_wallet_status())['result']['wallet_status']['is_synced_all'] != True:
        await asyncio.sleep(sleepy_time)  
    await does_it_go()


async def does_it_go(amount_pmob: int = 600000000) -> bool:
    network_status = await fs.get_network_status()
    assert "error" not in network_status.keys(),  "Failed to get network status"
    fee = int(network_status.get("result")
                            .get("network_status")
                            .get("fees")
                            .get("0")  # zero is the fee key for mob
    )

    """Test Setup """
    pmob_to_send = amount_pmob

    await preclean_this_test()

    alice = await get_account(0, "alice")
    bob = await get_account(1, "bob")

    await wait_for_account_to_sync(alice.id)
    
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
    await asyncio.sleep(sleepy_time)
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

    await test_cleanup()

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Basic test")
    parser.add_argument("config_path", nargs='?', type=str, default=default_config_path)
    args = parser.parse_args()

    with open(args.config_path) as json_file:
        config = json.load(json_file)

    asyncio.run(main())
