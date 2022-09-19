import argparse


def start(self):
    self.wallet_path.mkdir(parents=True, exists_ok=True)
    cmd = ' '.join([
        f'&& {constants.FULLSERVICE_DIR}/target/release/full-service',
        '--wallet-db /tmp/wallet-db/wallet.db',
        '--ledger-db /tmp/ledger-db/',
        '--peer insecure-mc://localhost:3200',
        '--peer insecure-mc://localhost:3201',
        f'--tx-source-url file://{constants.MOBILECOIN_DIR}/target/release/mc-local-network/node-ledger-distribution-0',
        f'--tx-source-url file://{constants.MOBILECOIN_DIR}/target/release/mc-local-network/node-ledger-distribution-1',
    ])
    print('===================================================')
    print('starting full service')
    print(cmd)
    self.full_service_process = subprocess.Popen(cmd, shell=True)

# retrieve accounts from mobilecoin/target/sample_data/keys
def get_test_accounts(self) -> Tuple[str, str]:
    print(f'retrieving accounts for account_keys_0 and account_keys_1')
    keyfile_0 = open(os.path.join(constants.KEY_DIR, 'account_keys_0.json'))
    keyfile_1 = open(os.path.join(constants.KEY_DIR, 'account_keys_1.json'))
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


