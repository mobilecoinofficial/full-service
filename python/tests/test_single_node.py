"""
Send transactions to each node individually, and check that they
completed successfully.
"""

from pathlib import Path
from subprocess import (
    Popen,
    PIPE,
)
import sys

from mobilecoin import Client, pmob2mob

NUM_TRANSACTIONS = 10


NODES='''
    mc://node1.test.mobilecoin.com/
    mc://node2.test.mobilecoin.com/
'''.strip().splitlines()

# NODES = '''
# mc://node1.prod.mobilecoinww.com/
# mc://node2.prod.mobilecoinww.com/
# mc://node3.prod.mobilecoinww.com:443/
# mc://blockdaemon.mobilecoin.bdnodes.net:443/
# mc://binance.mobilecoin.bdnodes.net:443/
# mc://ideasbeyondborders.mobilecoin.bdnodes.net:443/
# mc://thelongnowfoundation.mobilecoin.bdnodes.net:443/
# mc://node1.consensus.mob.production.namda.net:443/
# mc://node2.consensus.mob.production.namda.net:443/
# mc://ams1-mc-node1.dreamhost.com:3223/
# '''.strip().splitlines()

BASE_ENV = dict(
    MC_CHAIN_ID='test',
    MC_TX_SOURCE_URL='https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node1.test.mobilecoin.com/,https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node2.test.mobilecoin.com/',
    MC_FOG_INGEST_ENCLAVE_CSS='/home/christian/.mobilecoin/test/ingest-enclave.css',
    MC_WALLET_DB=str(Path.home() / '.mobilecoin/test/wallet-db/wallet.db'),
    MC_LEDGER_DB=str(Path.home() / '.mobilecoin/test/ledger-db'),
)

def main():
    fs_executable = sys.argv[1]
    account_id = sys.argv[2]

    # Go one node at a time, and send some transactions to each.
    # for node in NODES:
    for node in NODES:
        print('\n', '='*60, node, '='*60, '\n')

        # Run full-service
        proc = Popen(
            fs_executable,
            stdout=PIPE,
            stderr=PIPE,
            env=BASE_ENV | dict(MC_PEER=node)
        )

        # Initialize API.
        c = Client(verbose=True)
        c.poll(lambda: c.version(), delay=0.1)

        # Load and sync account.
        account = c.get_account(account_id)
        balance = c.poll_balance(account_id)

        # Send some transactions to ourselves.
        for i in range(10):
            print('\ntransaction #{}\n'.format(i+1))
            transaction_log = c.build_and_submit_transaction(account_id, 0.0004, account['main_address'])
            tx_index = int(transaction_log['submitted_block_index'])
            c.poll_balance(account_id, tx_index + 1)

        # Terminate full-service and record output.
        proc.terminate()

        print('\n', '='*60, '\n')
        print('Full-service stdout:\n')
        print(proc.stdout.read().decode())
        print('\n', '='*60, '\n')
        print('Full-service stderr:\n')
        print(proc.stderr.read().decode())


if __name__ == '__main__':
    main()
