import argparse
import asyncio 
import os
import json
import forest_utils as utils

def main():
    parser = argparse.ArgumentParser(description='Misc utils')
    parser.add_argument('--start', help='Start full service', action='store_true')
    parser.add_argument('--get-test-accounts', help='Get test accounts', action='store_true')
    parser.add_argument('--parse-network-type', help='Parse network type command line args', action='store_true')
    args = parser.parse_args()
    if args.start:
        print('starting full service')
        asyncio.run(start(self=None))
    elif args.get_test_accounts:
        print('getting test accounts')
        get_test_accounts(self=None)
    elif args.parse_network_type:
        print('parsing network type command line args')
        parse_network_type_cmd_line_args(self=None)

async def start(self):    
    #self.wallet_path.mkdir(parents=True, exists_ok=True)
    cmd = ' '.join([
        f'&& {utils.get_secret("FULLSERVICE_DIR")}/target/release/full-service',
        '--wallet-db /tmp/wallet-db/wallet.db',
        '--ledger-db /tmp/ledger-db/',
        '--peer insecure-mc://localhost:3200',
        '--peer insecure-mc://localhost:3201',
        f'--tx-source-url file://{utils.get_secret("MOBILECOIN_DIR")}/target/release/mc-local-network/node-ledger-distribution-0',
        f'--tx-source-url file://{utils.get_secret("MOBILECOIN_DIR")}/target/release/mc-local-network/node-ledger-distribution-1',
    ])
    full_service_process = await asyncio.create_subprocess_shell(cmd, stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE)
    stdout, stderr = await full_service_process.communicate()
    print(f'[{cmd!r} exited with {full_service_process.returncode}]')
    if stdout:
        print(f'[stdout]\n{stdout.decode()}')
    if stderr:
        print(f'[stderr]\n{stderr.decode()}')


# retrieve accounts from mobilecoin/target/sample_data/keys
def get_test_accounts(self) -> tuple[str, str]:
    print(f'retrieving accounts for account_keys_0 and account_keys_1')
    keyfile_0 = open(os.path.join(utils.get_secret("KEY_DIR"), 'account_keys_0.json'))
    keyfile_1 = open(os.path.join(utils.get_secret("KEY_DIR"), 'account_keys_1.json'))
    keydata_0 = json.load(keyfile_0)
    keydata_1 = json.load(keyfile_1)
    keyfile_0.close()
    keyfile_1.close()

    return (keydata_0['mnemonic'], keydata_1['mnemonic'])

# ensure at least two accounts are in the wallet. Some accounts are imported by default, but the number changes.
def setup_accounts(self):
    account_ids, account_map = self.get_all_accounts()
    if len(account_ids) >= 2:
        self.account_ids = account_ids
        self.account_map = account_map
    else:
        mnemonic_0, mnemonic_1 = self.get_test_accounts()
        self.import_account(mnemonic_0)
        self.import_account(mnemonic_1)

    self.account_ids, self.account_map = self.get_all_accounts()

def parse_network_type_cmd_line_args():
    # pull args from command line
    parser = argparse.ArgumentParser(description='Local network tester')
    parser.add_argument('--network-type', help='Type of network to create', required=True)
    parser.add_argument('--block-version', help='Set the block version argument', type=int)
    return parser.parse_args()


if __name__ == "__main__":
    main()
