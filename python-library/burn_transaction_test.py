import argparse
import asyncio
import json
import subprocess
import sys
from fullservice import FullServiceAPIv2 as v2
from FSDataObjects import Response, Account

repo_root_dir = (
    subprocess.check_output("git rev-parse --show-toplevel", shell=True)
    .decode("utf8")
    .strip()
)
sys.path.append(
    "{}/.internal-ci/test/fs-integration".format(repo_root_dir)
)  # we're importing the basic.py file as the integration test framework

import basic as itf
from basic import TestUtils as Utils

import export_check_all as account_tools  # this will be folded into the integration test framework

fs = v2()


async def test_burn_transaction(amount_pmob: int = 600000000):
    await Utils.wait_for_network_sync()
    # await account_tools.clean()
    Utils.get_mnemonics()
    alice = await itf.init_test_accounts(0, "alice", True)
    burn_tx = await fs.build_burn_transaction(
        alice.id,
        amount={"value": str(amount_pmob), "token_id": str(0)},
    )
    balance_before = int(
        (await fs.get_account_status(alice.id))
        .get("result")
        .get("balance_per_token")
        .get("0")
        .get("unspent")
    )

    #submitted_tx = await fs.submit_transaction(burn_tx.get("result").get("tx_proposal"))
    
    balance_after = int(
        (await fs.get_account_status(alice.id))
        .get("result")
        .get("balance_per_token")
        .get("0")
        .get("unspent")
    )

    assert balance_before < balance_after, "Burn transaction failed"
    




asyncio.run(test_burn_transaction())
