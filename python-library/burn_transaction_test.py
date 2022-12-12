import asyncio
import subprocess
import sys
from fullservice import FullServiceAPIv2 as v2
import pytest


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

    balance_before = int(
        (await fs.get_account_status(alice.id))
        .get("result")
        .get("balance_per_token")
        .get("0")
        .get("unspent")
    )

    burn_tx = await fs.build_burn_transaction(
        alice.id,
        amount={"value": str(amount_pmob), "token_id": str(0)},
    )

    print(type(burn_tx.get("result").get("tx_proposal")))

    await fs.submit_transaction(burn_tx.get("result").get("tx_proposal"))
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
