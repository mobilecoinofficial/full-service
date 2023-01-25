"""
Send transactions to each node individually, and check that they
completed successfully.
"""

import time
from pathlib import Path
from subprocess import (
    Popen,
    PIPE,
)
import sys

from mobilecoin import Client

NUM_TRANSACTIONS = 10

NODES = '''
mc://ideasbeyondborders.mobilecoin.bdnodes.net:443/
mc://thelongnowfoundation.mobilecoin.bdnodes.net:443/
mc://node1.consensus.mob.production.namda.net:443/
mc://node2.consensus.mob.production.namda.net:443/
mc://ams1-mc-node1.dreamhost.com:3223/
'''.strip().splitlines()

BASE_ENV = dict(
    MC_FOG_INGEST_ENCLAVE_CSS='/home/christian/.mobilecoin/test/ingest-enclave.css',
    MC_TX_SOURCE_URL='https://ledger.mobilecoinww.com/node1.prod.mobilecoinww.com/',
    MC_WALLET_DB=str(Path.home() / '.mobilecoin/main/wallet-db/wallet.db'),
    MC_LEDGER_DB=str(Path.home() / '.mobilecoin/main/ledger-db'),
)

def main():
    fs_executable = sys.argv[1]
    account_id = sys.argv[2]

    # Go one node at a time, and send some transactions to each.
    for node in NODES:
        print('\n' + '='*60 + '\n', node, '\n' + '='*60 + '\n')

        # Run full-service
        proc = Popen(
            [
                fs_executable,
                # We have to specify these on the command line in v1.9.6.
                '--tx-source-url=' + BASE_ENV['MC_TX_SOURCE_URL'],
                '--ledger-db=' + BASE_ENV['MC_LEDGER_DB'],
                '--wallet-db=' + BASE_ENV['MC_WALLET_DB'],
                '--peer=' + node,
            ],
            stdout=PIPE,
            stderr=PIPE,
            env=BASE_ENV | dict(MC_PEER=node)
        )

        # Initialize API.
        c = Client(verbose=True)
        c.poll(lambda: c.version(), delay=0.1)

        # Load and sync account.
        account = c.get_account(account_id)
        try:
            balance = c.poll_balance(account_id)
        except TimeoutError:
            pass

        # Send some transactions to ourselves.
        for i in range(NUM_TRANSACTIONS):
            print('\ntransaction #{}\n'.format(i+1))
            transaction_log = c.build_and_submit_transaction(account_id, 0.0004, account['main_address'])
            tx_index = int(transaction_log['submitted_block_index'])
            c.poll_balance(account_id, tx_index + 1, delay=0.3)

        # Terminate full-service and record output.
        proc.terminate()

        print('\n' + '='*60 + '\n')
        print('Full-service stdout:\n')
        print(proc.stdout.read().decode())
        print('\n' + '='*60 + '\n')
        print('Full-service stderr:\n')
        print(proc.stderr.read().decode())


if __name__ == '__main__':
    main()
