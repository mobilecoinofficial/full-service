import asyncio
import subprocess
import sys
from fullservice import FullServiceAPIv2 as v2
import pytest
import transaction_signer_lib as signer
import json

repo_root_dir = (
    subprocess.check_output("git rev-parse --show-toplevel", shell=True)
    .decode("utf8")
    .strip()
)
sys.path.append(
    "{}/.internal-ci/test/fs-integration".format(repo_root_dir)
)  # we're importing the basic.py file as the integration test framework

from basic import TestUtils as Utils

fs = v2()


@pytest.mark.asyncio
async def test_burn_transaction(amount_pmob: int = 600000000):
    utils = Utils()
    Utils.get_mnemonics()
    alice = await utils.init_test_accounts(0, "alice", True)
    alice_export = await fs.export_account_secrets(alice.id)
    entropy = (
        alice_export.get("result")
        .get("account_secrets")
        .get("mnemonic")
        .removeprefix("(")
        .removesuffix(")")
    )  # clean up entropy response
    signer.create_account(name="alice", mnemonic=entropy)
    balance_before = int(
        (await fs.get_account_status(alice.id))
        .get("result")
        .get("balance_per_token")
        .get("0")
        .get("unspent")
    )

    # write the unsigned transaction request to a file
    unsigned_burn_tx_request = await fs.build_unsigned_burn_transaction(
        alice.id,
        amount={"value": str(amount_pmob), "token_id": str(0)},
    )
    to_json = json.dumps(unsigned_burn_tx_request.get("result"), indent=4)
    with open("transaction_request.json", "w") as outfile:
        outfile.write(to_json)

    # get id for mnemonic file name and sign transaction
    id = alice.id[0:6]
    signer.sign_transaction(
        secret_mnemonic=f"mobilecoin_secret_mnemonic_{id}.json",
        sign_request="transaction_request.json",
    )

    # get the transaction request and submit it
    with open("transaction_request.json_completed.json", "r") as infile:
        signed_tx: dict = json.load(infile)
    tx_proposal = signed_tx.get("params").get("tx_proposal")
    await fs.submit_transaction(tx_proposal)

    await utils.wait_two_blocks()
    balance_after = int(
        (await fs.get_account_status(alice.id))
        .get("result")
        .get("balance_per_token")
        .get("0")
        .get("unspent")
    )
    print(balance_before, balance_after)
    assert balance_before > balance_after, "Burn transaction failed"
    exit


asyncio.run(test_burn_transaction())
