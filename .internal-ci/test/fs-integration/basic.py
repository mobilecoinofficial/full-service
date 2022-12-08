import argparse
import asyncio
import json
import subprocess
import sys

repo_root_dir = (
    subprocess.check_output("git rev-parse --show-toplevel", shell=True)
    .decode("utf8")
    .strip()
)
sys.path.append("{}/python-library".format(repo_root_dir))

from fullservice import FullServiceAPIv2 as v2
from FSDataObjects import Response, Account


sleepy_time = 15
default_config_path = "./test_config.json"

account_ids = []
config = []

fs = v2()


class TestUtils:
    # may be scuffed 
    async def wait_for_account_sync():
        print("Checking if all accounts are synced...")
        initial_response = str(
            (await fs.get_wallet_status())
            .get("result")
            .get("wallet_status")
            .get("is_synced_all")
        )
        if initial_response == "True":
            print("Accounts are synced!")
            return
        else:
            while str(initial_response) != "True":
                await asyncio.sleep(15)
                print("Waiting for accounts to sync...")
                if str(
                        (await fs.get_wallet_status())
                        .get("result")
                        .get("wallet_status")
                        .get("is_synced_all")
                    ) == "True":
                        print("Accounts are synced!")
                        break
                

    async def wait_two_blocks():
        print("Waiting for next block")
        # network_status = await fs.get_network_status()
        starting_block_height = int(
            (await fs.get_network_status())
            .get("result")
            .get("network_status")
            .get("local_block_height")
        )
        print("starting Height:", starting_block_height)
        while int(
                (await fs.get_network_status())
                .get("result")
                .get("network_status")
                .get("local_block_height")
            ) <= starting_block_height + 1:
            await asyncio.sleep(5)
            print("Waiting for next block...")
            

    async def clean():
        """This function will remove all of your accounts!! Be careful!"""
        print("Cleaning up accounts")
        accounts = await fs.get_accounts()
        result = accounts.get("result").get("account_ids")
        for account_id in result:
            await fs.remove_account(account_id)
            return print("Done cleaning up accounts")

    # If this test fails before reaching the last cleanup step, we have leftover
    # artifacts in the FS instance. We clean up those residual artifacts here.
    # Note: Using a testing framework like pytest would allow us to bundle this in
    # a test fixture
    async def preclean_this_test():
        await TestUtils.init_test_accounts(0, "alice", True)
        await TestUtils.init_test_accounts(1, "bob", True)

    def get_mnemonics(n=2):
        if n > len(config["Account Mnemonics"]):
            raise ValueError("Not enough account available in config")
        return config["Account Mnemonics"][:n]

    # test functions should be named differently from FS endpoint functions 
    async def init_test_accounts(index, name="", already_imported=False):
        print(index)
        mnemonic = config["Account Mnemonics"][index]["mnemonic"]
        account = await fs.import_account(
            mnemonic,
            "2",  # This parameter indicates that we are using the 2nd key derivations method (mnemonics)
            name=name,
        )

        if not already_imported:
            assert "error" not in account.keys(), "Failed to import account"

        # Newly imported
        if "error" not in account.keys():
            result = Account(account["result"]["account"])
            account_ids.append(result.id)
        # Previously imported
        else:
            error_msg = account.get("error").get("data").get("details")
            assert error_msg.startswith(
                "Error interacting& with the database: Account already exists:"
            ), "Unknown import failure"
            id = error_msg.split()[-1]
            result = Response(await fs.get_account_status(id)).account
            account_ids.append(result.id)
        return result


async def main():
    while (await fs.get_wallet_status())["result"]["wallet_status"][
        "is_synced_all"
    ] != True:
        await asyncio.sleep(sleepy_time)
    await does_it_go()


async def does_it_go(amount_pmob: int = 600000000) -> bool:
    network_status = await fs.get_network_status()
    assert "error" not in network_status.keys(), "Failed to get network status"
    fee = int(
        network_status.get("result")
        .get("network_status")
        .get("fees")
        .get("0")  # zero is the fee key for mob
    )

    """Test Setup """
    pmob_to_send = amount_pmob

    # await preclean_this_test()

    alice = await TestUtils.init_test_accounts(0, "alice", True)
    bob = await TestUtils.init_test_accounts(1, "bob", True)

    await TestUtils.wait_for_account_sync()
    

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

    first_transaction = await fs.build_and_submit_transaction(
        alice.id,
        recipient_public_address=bob.main_address,
        amount={"value": str(pmob_to_send), "token_id": str(0)},
    )

    # TODO: replace this with a poll loop that waits a block or two
    await TestUtils.wait_two_blocks()

    """ Check Results """
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

    assert (
        alice_balance_0 == alice_balance_1 + fee + pmob_to_send
    ), "Alice doesn't end with the expected amount"
    assert (
        bob_balance_1 == bob_balance_0 + pmob_to_send
    ), "Bob doesn't end with the expected amount"
    
    print("Test passed")
    
    # await test_cleanup()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Basic test")
    parser.add_argument("config_path", nargs="?", type=str, default=default_config_path)
    args = parser.parse_args()
    with open(default_config_path) as json_file:
        config = json.load(json_file)
    asyncio.run(main())

if __name__ not in ["__main__", "__builtin__"]:
    with open(default_config_path) as json_file:
        config = json.load(json_file)
