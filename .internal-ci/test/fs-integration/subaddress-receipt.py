import asyncio
import sys
import subprocess
import json
import argparse
repo_root_dir = subprocess.check_output("git rev-parse --show-toplevel", shell=True).decode("utf8").strip()
sys.path.append("{}/python-library".format(repo_root_dir))

from fullservice import FullServiceAPIv2 as v2
from basic import wait_for_account_to_sync
from FSDataObjects import Response, Account 


async def main(config):
    # General test setup
    fs = v2()

    while (await fs.get_wallet_status())['result']['wallet_status']['is_synced_all'] != True:
        await asyncio.sleep(sleepy_time)  
    network_status = await fs.get_network_status()
    fee = int(network_status.get("result")
                                .get("network_status")
                                .get("fees")
                                .get("0")  # zero is the fee key for mob
        )

    mnemonic = config["Account Mnemonics"][0]["mnemonic"]
    account = await fs.import_account(
        mnemonic,
        "2",  # This parameter indicates that we are using the 2nd key derivations method (mnemonics)
        name="alice"
    )
    alice = Account(account["result"]["account"])
    
    mnemonic = config["Account Mnemonics"][1]["mnemonic"]
    account = await fs.import_account(
        mnemonic,
        "2",  # This parameter indicates that we are using the 2nd key derivations method (mnemonics)
        name="bob"
    )
    bob = Account(account["result"]["account"])

    await wait_for_account_to_sync(alice.id)
    await wait_for_account_to_sync(bob.id)

    await wait_for_account_to_sync(alice.id)
    await wait_for_account_to_sync(bob.id)

    pmob_to_send = 600000000

    # Initial conditions
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

    # do
    response = await fs.assign_address_for_account(bob.id, "Order #600: Birthday Cake")
    bob_subaddr = response['result']['address']['public_address_b58']
    response = await fs.get_address_status(bob_subaddr)
    assert response.get("result").get("balance_per_token").get("0").get("unspent") == "0"

    response = await fs.build_and_submit_transaction(alice.id,
                                                recipient_public_address=bob_subaddr,
                                                amount = {"value": str(pmob_to_send),
                                                        "token_id":str(0)
                                                        }
                                                )
    await wait_for_account_to_sync(bob.id)
    response = await fs.get_address_status(bob_subaddr)
    
    assert response.get("result").get("balance_per_token").get("0").get("unspent") == str(pmob_to_send)
    

    # Cleanup
    # TBD


if __name__ == "__main__":
    default_config_path = "./test_config.json"
    parser = argparse.ArgumentParser(description="Basic test")
    parser.add_argument("config_path", nargs='?', type=str, default=default_config_path)
    args = parser.parse_args()

    with open(args.config_path) as json_file:
        config = json.load(json_file)

    asyncio.run(main(config))
