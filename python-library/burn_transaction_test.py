import argparse
import asyncio
import json
import subprocess
import sys
from fullservice import FullServiceAPIv2 as v2
from FSDataObjects import Response, Account 

repo_root_dir = subprocess.check_output("git rev-parse --show-toplevel", shell=True).decode("utf8").strip()
sys.path.append("{}/.internal-ci/test/fs-integration".format(repo_root_dir))

import basic as itf # import basic as the integration test framework. this should live in a different file.
from basic import TestUtils as Utils
import export_check_all as account_tools

fs = v2()

async def import_accounts():
    Utils.get_mnemonics()
    alice = await itf.init_test_accounts(0, "alice", True)
    bob = await itf.init_test_account(1, "bob", True)
    burn_tx = await fs.build_burn_transaction(alice.id, "400")

async def test_burn_transaction():
    await import_accounts()
   


asyncio.run(test_burn_transaction())
