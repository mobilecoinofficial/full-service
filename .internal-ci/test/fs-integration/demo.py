import argparse
import asyncio
import json
import subprocess
import sys

repo_root_dir = subprocess.check_output("git rev-parse --show-toplevel", shell=True).decode("utf8").strip()
sys.path.append("{}/python-library".format(repo_root_dir))

from fullservice import FullServiceAPIv2 as v2
from FSDataObjects import Response, Account 

demo_time = True
sleepy_time = 3
config_path = "./test_config.json"
mnemonics=[]
account_ids = []

fs = v2()

PICO_MOB= 1000000000000
def demo(*args):
    if demo_time:
        input(*args)


async def run_demo():
    global demo_time
    demo_time = False

    demo("Welcome to Alice's journey with full-service \n--------------------------------------------")
    demo("Alice is already using the Moby app. As the most tech savvy person in her family, she's decided to use \nfull-service to help manage her loved ones' wallets.")
    demo("Earlier today, Alice followed the installation instructions to setup and run full-serivce.")
    demo("She already has a wallet with some funds, and will use her mnemonic to setup the wallet on full-service too.")

    demo("Before she imports her account, Alice wants to make sure her full-service wallet is synced with the network.")
    demo("She uses her rest client to call the get_wallet_status API.")
    
    while True:
        demo(f"Sending request to get_wallet_status ...")
        response = await fs.get_wallet_status()
        synced = response['result']['wallet_status']['is_synced_all']
        demo(f"get_wallet_status tells us the network is{' not' if not synced else ''} synced")
        if (synced):
            break
        else:
            demo(f"We'll wait {sleepy_time} seconds and try again")
            await asyncio.sleep(sleepy_time)
        
    demo("Awesome! Now that the wallet is synced, Alice wants to import some accounts.")

    demo("Alice starts by importing her own account by passing in her mnemonic to the import_account API.")
    demo("Sending request to import_account ...")
    response = await fs.import_account(mnemonics[0], "2")
    await asyncio.sleep(sleepy_time)
    alice_id = response['result']['account']['id']
    alice_addr = response['result']['account']['main_address']
    demo("Let's look at the request output: ")
    demo(json.dumps(response, indent=4))

    demo("Oh no, Alice forgot to specify the name field. Since there will be several individuals with accounts on \nthis wallet, Alice wants to make sure its easy to tell whose account is whose")
    demo("Luckily, she can update the account name with another API. She does this now.")
    demo("Sending request to update_account_name ...")
    response = await fs.update_account_name(account_id=alice_id, name="Alice")
    demo("Can we see her name on the account object now?")
    demo(json.dumps(response, indent=4))
    demo("Great, looks like it's there. Now Alice will import wallets for other loved ones.")


    demo("Sending request to import Mom's account ...")
    response = await fs.import_account(mnemonics[1], "2", "Mom")
    await asyncio.sleep(sleepy_time)
    demo("Sending request to import Chadicus's account ...")
    response = await fs.import_account(mnemonics[2], "2", "Chadicus")
    await asyncio.sleep(sleepy_time)
    chad_id = response['result']['account']['id']
    demo("Alice is done importing accounts.")

    demo("Sending request to get_accounts...")
    response = await fs.get_accounts()
    demo("Looking at the get_accounts_API, we see the wallet now have 3 accounts.")
    demo(json.dumps(response, indent=4))

    demo("Alice and her partner, Chadicus, have decided to part ways. Alice will no longer manage their wallet as \npart of her family Full-Service instance.")
    demo("Alice will export the account secrets to hand off to Chadicus so he can manage his own wallet.")
    demo("Sending request to export_account_secrets ...")
    response = await fs.export_account_secrets(chad_id)
    demo("Shh ... here are Chadicus's secrets:")
    demo(json.dumps(response, indent=4))
    demo("It's time to say final good byes to Chadicus's account. Alice removes it from full-service")
    demo("Sending request to remove_account...")
    response = await fs.remove_account(chad_id)
    demo(f"Looking at the accounts API, we see Full-Service now has 2 accounts: Alice's and Mom's \n{response}")
    demo("Sending request to get_accounts ...")
    response = await fs.get_accounts()
    demo(json.dumps(response, indent=4))

    demo("".join(["Alice wants to gift her brother, Bob, some mob for his 16th birthday.", 
        "\nHe's never used mob before, so he doesn't have a wallet for her to send funds to.", 
        "\nAlice can use full service to make him an account and send him mob"]))
    demo("Sending request to create_account...")
    response = await fs.create_account("Bro Bob")
    bob_id = response['result']['account']['id']
    bob_addr = response['result']['account']['main_address']
    demo("Here's the response:")
    demo(json.dumps(response, indent=4))

    demo("Now that Bob has an account, Alice can send him some mob")
    response = await fs.get_account_status(alice_id)
    response = await fs.get_account_status(bob_id)
    response = await fs.build_and_submit_transaction(alice_id,
                                                    recipient_public_address=bob_addr,
                                                    amount = {"value": str(10 * PICO_MOB),
                                                            "token_id":str(0)
                                                            }
                                                    )
    await asyncio.sleep(sleepy_time*3)
    demo("If we check Bob's account status, we see that he now has 10 mob.")
    demo("Sending request to get_account_status...")
    response = await fs.get_account_status(bob_id)
    demo(json.dumps(f"{response['result']['balance_per_token']['0']}", indent=4))

    demo_time = True
    demo("Alice's home bakery business accepts mob for payment. She uses the subaddress feature to keep track of \nwhich payments are associated with which order")
    demo("Alice creates a new subaddress and associates it with Order #600")
    demo("Sending request to assign_address_for_account...")
    response = await fs.assign_address_for_account(alice_id, "Order #600: Birthday Cake")
    print(response)
    alice_subaddr = response['result']['address']['public_address_b58']
    demo(f"Alice's main address is {alice_addr}, but she shares this subaddress with her client: \n{alice_subaddr}")
    demo(f"She can use the verify_address API to double check the subaddr she saved is associated with an account on \nher full-service instance")
    demo(f"Sending request to verify_address...")
    response = await fs.verify_address(alice_subaddr)
    demo(json.dumps(response, indent=4))
    demo(f"Just to check, let's make sure no funds were received at this subbaddress yet")
    demo(f"Sending request to get_address_status...")
    response = await fs.get_address_status(alice_subaddr)
    demo(json.dumps(response, indent=4))
    demo(f"As we can see, the subbaddress hasn't received any funds yet:")


    # have bob send her money to the subaddress
    response = await fs.build_and_submit_transaction(bob_id,
                                                recipient_public_address=alice_subaddr,
                                                amount = {"value": str(5 * PICO_MOB),
                                                        "token_id":str(0)
                                                        }
                                                )
    await asyncio.sleep(3*sleepy_time)
    demo("Alice's client claims they sent over the funds ... Let's check that.")
    demo("Sending request to get_address_status...")
    response = await fs.get_address_status(alice_subaddr)
    demo(json.dumps(response, indent=4))


if __name__ == "__main__":
    with open(config_path) as json_file:
        mnemonics = json.load(json_file)
        mnemonics = [x["mnemonic"] for x in mnemonics["Account Mnemonics"]]
        

    asyncio.run(run_demo())