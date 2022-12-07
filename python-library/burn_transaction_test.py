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
    # await account_tools.clean()
    Utils.get_mnemonics()
    alice = await itf.init_test_accounts(0, "alice", True)
    bob = await itf.init_test_accounts(1, "bob", True)
    burn_tx = await fs.build_and_submit_transaction(
        alice.id,
        recipient_public_address=bob.main_address,
        amount={"value": str(amount_pmob), "token_id": str(0)},
    )

    await Utils.wait_for_network_sync()


asyncio.run(test_burn_transaction())
