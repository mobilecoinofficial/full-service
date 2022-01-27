import argparse
from decimal import Decimal
from getpass import getpass
import json
import os
from pathlib import Path
import subprocess
from textwrap import indent

from .client import (
    Client, WalletAPIError,
    MAX_TOMBSTONE_BLOCKS,
    pmob2mob,
)


class CommandLineInterface:

    def __init__(self):
        self.verbose = False
        self.config = json.loads(os.environ['MOBILECOIN_CONFIG'])

    def main(self):
        self._create_parsers()

        args = self.parser.parse_args()
        args = vars(args)
        command = args.pop('command')
        if command is None:
            self.parser.print_help()
            exit(1)

        self.verbose = args.pop('verbose')
        self.auto_confirm = args.pop('yes')

        self.client = Client(url=self.config.get('api-url'), verbose=self.verbose)

        # Dispatch command.
        setattr(self, 'import', self.import_)  # Can't name a function "import".
        command = command.translate(str.maketrans('-', '_'))
        command_func = getattr(self, command)
        try:
            command_func(**args)
        except ConnectionError as e:
            print(e)
            print('Did you run "mobcli start"? You may also want to check the logs at {}.'.format(self.config['logfile']))
            exit(1)

    def _create_parsers(self):
        self.parser = argparse.ArgumentParser(
            prog='mobilecoin',
            description='MobileCoin command-line wallet.',
        )
        self.parser.add_argument('-v', '--verbose', action='store_true', help='Show more information.')
        self.parser.add_argument('-y', '--yes', action='store_true', help='Do not ask for confirmation.')

        command_sp = self.parser.add_subparsers(dest='command', help='Commands')

        # Start server.
        self.start_args = command_sp.add_parser('start', help='Start the local MobileCoin wallet server.')
        self.start_args.add_argument('--offline', action='store_true', help='Start in offline mode.')
        self.start_args.add_argument('--bg', action='store_true',
                                     help='Start server in the background, stop with "mobilecoin stop".')
        self.start_args.add_argument('--unencrypted', action='store_true',
                                     help='Do not encrypt the wallet database. Secret keys will be stored on the hard drive in plaintext.')
        self.start_args.add_argument('--change-password', action='store_true',
                                     help='Change the password for the database.')

        # Stop server.
        self.stop_args = command_sp.add_parser('stop', help='Stop the local MobileCoin wallet server.')

        # Network status.
        self.status_args = command_sp.add_parser('status', help='Check the status of the MobileCoin network.')

        # List accounts.
        self.list_args = command_sp.add_parser('list', help='List accounts.')

        # Create account.
        self.create_args = command_sp.add_parser('create', help='Create a new account.')
        self.create_args.add_argument('-n', '--name', help='Account name.')

        # Rename account.
        self.rename_args = command_sp.add_parser('rename', help='Change account name.')
        self.rename_args.add_argument('account_id', help='ID of the account to rename.')
        self.rename_args.add_argument('name', help='New account name.')

        # Import account.
        self.import_args = command_sp.add_parser('import', help='Import an account.')
        self.import_args.add_argument('backup', help='Account backup file, mnemonic recovery phrase, or legacy root entropy in hexadecimal.')
        self.import_args.add_argument('-n', '--name', help='Account name.')
        self.import_args.add_argument('-b', '--block', type=int,
                                      help='Block index at which to start the account. No transactions before this block will be loaded.')
        self.import_args.add_argument('--key_derivation_version', type=int, default=2,
                                      help='The version number of the key derivation path which the mnemonic was created with.')

        # Export account.
        self.export_args = command_sp.add_parser('export', help='Export secret entropy mnemonic.')
        self.export_args.add_argument('account_id', help='ID of the account to export.')
        self.export_args.add_argument('-s', '--show', action='store_true',
                                      help='Only show the secret entropy mnemonic, do not write it to file.')

        # Remove account.
        self.remove_args = command_sp.add_parser('remove', help='Remove an account from local storage.')
        self.remove_args.add_argument('account_id', help='ID of the account to remove.')
        # Show transaction history.
        self.history_args = command_sp.add_parser('history', help='Show account transaction history.')
        self.history_args.add_argument('account_id', help='Account ID.')

        # Send transaction.
        self.send_args = command_sp.add_parser('send', help='Send a transaction.')
        self.send_args.add_argument('--build-only', action='store_true', help='Just build the transaction, do not submit it.')
        self.send_args.add_argument('--fee', type=str, default=None, help='The fee paid to the network.')
        self.send_args.add_argument('account_id', help='Source account ID.')
        self.send_args.add_argument('amount', help='Amount of MOB to send.')
        self.send_args.add_argument('to_address', help='Address to send to.')

        # Submit transaction proposal.
        self.submit_args = command_sp.add_parser('submit', help='Submit a transaction proposal.')
        self.submit_args.add_argument('proposal', help='A tx_proposal.json file.')
        self.submit_args.add_argument('account_id', nargs='?', help='Source account ID. Only used for logging the transaction.')
        self.submit_args.add_argument('--receipt', action='store_true', help='Also create a receiver receipt for the transaction.')

        # Address QR code.
        self.qr_args = command_sp.add_parser('qr', help='Show account address as a QR code')
        self.qr_args.add_argument('account_id', help='Account ID.')

        # Address commands.
        self.address_args = command_sp.add_parser('address', help='Account receiving address commands.')
        address_action = self.address_args.add_subparsers(dest='action')

        # List addresses.
        self.address_list_args = address_action.add_parser('list', help='List addresses and balances for an account.')
        self.address_list_args.add_argument('account_id', help='Account ID.')

        # Create address.
        self.address_create_args = address_action.add_parser(
            'create',
            help='Create a new receiving address for the specified account.',
        )
        self.address_create_args.add_argument('account_id', help='Account ID.')
        self.address_create_args.add_argument('metadata', nargs='?', help='Address label.')

        # Gift code commands.
        self.gift_args = command_sp.add_parser('gift', help='Gift code commands.')
        gift_action = self.gift_args.add_subparsers(dest='action')

        # List gift codes.
        self.gift_list_args = gift_action.add_parser('list', help='List gift codes and their amounts.')

        # Create gift code.
        self.gift_create_args = gift_action.add_parser('create', help='Create a new gift code.')
        self.gift_create_args.add_argument('account_id', help='Source account ID.')
        self.gift_create_args.add_argument('amount', help='Amount of MOB to add to the gift code.')
        self.gift_create_args.add_argument('-m', '--memo', help='Gift code memo.')

        # Claim gift code.
        self.gift_claim_args = gift_action.add_parser('claim', help='Claim a gift code, adding the funds to your account.')
        self.gift_claim_args.add_argument('account_id', help='Destination account ID to deposit the gift code funds.')
        self.gift_claim_args.add_argument('gift_code', help='Gift code string')

        # Remove gift code.
        self.gift_remove_args = gift_action.add_parser('remove', help='Remove a gift code.')
        self.gift_remove_args.add_argument('gift_code', help='Gift code to remove.')

    def _load_account_prefix(self, prefix):
        accounts = self.client.get_all_accounts()
        matching_ids = [
            a_id for a_id in accounts.keys()
            if a_id.startswith(prefix)
        ]
        if len(matching_ids) == 0:
            print('Could not find account starting with', prefix)
            exit(1)
        elif len(matching_ids) == 1:
            account_id = matching_ids[0]
            return accounts[account_id]
        else:
            print('Multiple matching matching ids: {}'.format(', '.join(matching_ids)))
            exit(1)

    def confirm(self, message):
        if self.auto_confirm:
            return True
        confirmation = input(message)
        return confirmation.lower() in ['y', 'yes']

    def start(self, offline=False, bg=False, unencrypted=False, change_password=False):
        password = ''
        new_password = ''
        if not unencrypted:
            password = getpass('Wallet database password: ')
            if password == '':
                print('You must provide a password, or start the server with the option "--unencrypted".')
                exit(1)
            if change_password:
                new_password = getpass('New password: ')
                if new_password == '':
                    print('Cannot change to an empty password.')
                    exit(1)
                confirm_password = getpass('Confirm new password: ')
                if new_password != confirm_password:
                    print('Passwords do not match.')
                    exit(1)

        env = dict(os.environ)
        env['MC_PASSWORD'] = password
        if new_password != '':
            env['MC_CHANGED_PASSWORD'] = new_password

        wallet_server_command = [
            self.config['executable'],
            '--ledger-db', self.config['ledger-db'],
            '--wallet-db', self.config['wallet-db'],
        ]
        if offline:
            wallet_server_command += ['--offline']
        else:
            for peer in self.config['peer']:
                wallet_server_command += ['--peer', peer]
            for tx_source_url in self.config['tx-source-url']:
                wallet_server_command += ['--tx-source-url', tx_source_url]

        ingest_enclave = self.config.get('fog-ingest-enclave-css')
        if ingest_enclave:
            wallet_server_command += ['--fog-ingest-enclave-css', ingest_enclave]

        if bg:
            wallet_server_command += [
                '>', self.config['logfile'], '2>&1'
            ]

        if self.verbose:
            print(' '.join(wallet_server_command))

        print('Starting {}...'.format(Path(self.config['executable']).name))

        Path(self.config['ledger-db']).mkdir(parents=True, exist_ok=True)
        Path(self.config['wallet-db']).parent.mkdir(parents=True, exist_ok=True)

        if bg:
            subprocess.Popen(' '.join(wallet_server_command), shell=True, env=env)
            print('Started, view log at {}.'.format(self.config['logfile']))
            print('Stop server with "mobcli stop".')
        else:
            subprocess.run(' '.join(wallet_server_command), shell=True, env=env)

    def stop(self):
        if self.verbose:
            print('Stopping MobileCoin wallet server...')
        subprocess.Popen(['killall', '-v', self.config['executable']])

    def status(self):
        network_status = self.client.get_network_status()
        fee = pmob2mob(network_status['fee_pmob'])

        if int(network_status['network_block_height']) == 0:
            print('Offline.')
            print('Local ledger has {} blocks.'.format(network_status['local_block_height']))
            print('Expected fee is {}'.format(_format_mob(fee)))
        else:
            print('Connected to network.')
            print('Local ledger has {}/{} blocks.'.format(
                network_status['local_block_height'],
                network_status['network_block_height'],
            ))
            print('Network fee is {}'.format(_format_mob(fee)))

    def list(self, **args):
        accounts = self.client.get_all_accounts(**args)

        if len(accounts) == 0:
            print('No accounts.')
            return

        account_list = []
        for account_id, account in accounts.items():
            balance = self.client.get_balance_for_account(account_id)
            account_list.append((account_id, account, balance))

        for (account_id, account, balance) in account_list:
            print()
            _print_account(account, balance)

        print()

    def create(self, **args):
        account = self.client.create_account(**args)
        print('Created a new account.')
        print()
        _print_account(account)
        print()

    def rename(self, account_id, name):
        account = self._load_account_prefix(account_id)
        old_name = account['name']
        account_id = account['account_id']
        account = self.client.update_account_name(account_id, name)
        print('Renamed account from "{}" to "{}".'.format(
            old_name,
            account['name'],
        ))
        print()
        _print_account(account)
        print()

    def import_(self, backup, name=None, block=None, key_derivation_version=2):
        data = _load_import(backup)

        if name is not None:
            data['name'] = name
        if block is not None:
            data['first_block_index'] = block

        if 'mnemonic' in data:
            data['key_derivation_version'] = key_derivation_version
            account = self.client.import_account(**data)
        elif 'legacy_root_entropy' in data:
            account = self.client.import_account_from_legacy_root_entropy(**data)
        else:
            raise ValueError('Could not import account from {}'.format(backup))

        print('Imported account.')
        print()
        _print_account(account)
        print()

    def export(self, account_id, show=False):
        account = self._load_account_prefix(account_id)
        account_id = account['account_id']
        balance = self.client.get_balance_for_account(account_id)

        print('You are about to export the secret entropy mnemonic for this account:')
        print()
        _print_account(account, balance)

        print()
        if show:
            print('The entropy will display on your screen. Make sure your screen is not being viewed or recorded.')
        else:
            print('Keep the exported entropy file safe and private!')
        print('Anyone who has access to the entropy can spend all the funds in the account.')

        if show:
            confirm_message = 'Really show account entropy mnemonic? (Y/N) '
        else:
            confirm_message = 'Really write account entropy mnemonic to a file? (Y/N) '
        if not self.confirm(confirm_message):
            print('Cancelled.')
            return

        secrets = self.client.export_account_secrets(account_id)
        if show:
            mnemonic_words = secrets['mnemonic'].upper().split()
            print()
            for i, word in enumerate(mnemonic_words, 1):
                print('{:<2}  {}'.format(i, word))
            print()
        else:
            filename = 'mobilecoin_secret_entropy_{}.json'.format(account_id[:16])
            try:
                _save_export(account, secrets, filename)
            except OSError as e:
                print('Could not write file: {}'.format(e))
                exit(1)
            else:
                print(f'Wrote {filename}.')

    def remove(self, account_id):
        account = self._load_account_prefix(account_id)
        account_id = account['account_id']
        balance = self.client.get_balance_for_account(account_id)

        print('You are about to remove this account:')
        print()
        _print_account(account, balance)
        print()
        print('You will lose access to the funds in this account unless you')
        print('restore it from the mnemonic phrase.')
        if not self.confirm('Continue? (Y/N) '):
            print('Cancelled.')
            return

        self.client.remove_account(account_id)
        print('Removed.')

    def history(self, account_id):
        account = self._load_account_prefix(account_id)
        account_id = account['account_id']

        transactions = self.client.get_all_transaction_logs_for_account(account_id)

        def block_key(t):
            submitted = t['submitted_block_index']
            finalized = t['finalized_block_index']
            if submitted is not None and finalized is not None:
                return min([submitted, finalized])
            elif submitted is not None and finalized is None:
                return submitted
            elif submitted is None and finalized is not None:
                return finalized
            else:
                return None

        transactions = sorted(transactions.values(), key=block_key)

        for t in transactions:
            print()
            if t['direction'] == 'tx_direction_received':
                amount = _format_mob(
                    sum(
                        pmob2mob(txo['value_pmob'])
                        for txo in t['output_txos']
                    )
                )
                print('Received {}'.format(amount))
                print('  at {}'.format(t['assigned_address_id']))
            elif t['direction'] == 'tx_direction_sent':
                for txo in t['output_txos']:
                    amount = _format_mob(pmob2mob(txo['value_pmob']))
                    print('Sent {}'.format(amount))
                    if not txo['recipient_address_id']:
                        print('  to an unknown address.')
                    else:
                        print('  to {}'.format(txo['recipient_address_id']))
            print('  in block {}'.format(block_key(t)), end=', ')
            if t['fee_pmob'] is None:
                print('paying an unknown fee.')
            else:
                print('paying a fee of {}'.format(_format_mob(pmob2mob(t['fee_pmob']))))
        print()

    def send(self, account_id, amount, to_address, build_only=False, fee=None):
        account = self._load_account_prefix(account_id)
        account_id = account['account_id']
        balance = self.client.get_balance_for_account(account_id)
        unspent = pmob2mob(balance['unspent_pmob'])

        network_status = self.client.get_network_status()

        if fee is None:
            fee = pmob2mob(network_status['fee_pmob'])
        else:
            fee = Decimal(fee)

        if unspent <= fee:
            print('There is not enough MOB in account {} to pay the transaction fee.'.format(account_id[:6]))
            return

        if amount == "all":
            amount = unspent - fee
            total_amount = unspent
        else:
            amount = Decimal(amount)
            total_amount = amount + fee

        if build_only:
            verb = 'Building transaction for'
        else:
            verb = 'Sending'

        print('\n'.join([
            '{} {} from account {} {}',
            'to address {}',
            'Fee is {}, for a total amount of {}.',
        ]).format(
            verb,
            _format_mob(amount),
            account_id[:6],
            account['name'],
            to_address,
            _format_mob(fee),
            _format_mob(total_amount),
        ))

        if total_amount > unspent:
            print('\n'.join([
                'Cannot send this transaction, because the account only',
                'contains {}. Try sending all funds by entering amount as "all".',
            ]).format(_format_mob(unspent)))
            return

        if build_only:
            tx_proposal = self.client.build_transaction(account_id, amount, to_address, fee=fee)
            path = Path('tx_proposal.json')
            if path.exists():
                print(f'The file {path} already exists. Please rename the existing file and retry.')
            else:
                with path.open('w') as f:
                    json.dump(tx_proposal, f, indent=2)
                print(f'Wrote {path}')
            return

        if not self.confirm('Confirm? (Y/N) '):
            print('Cancelled.')
            return

        transaction_log, tx_proposal = self.client.build_and_submit_transaction_with_proposal(
            account_id,
            amount,
            to_address,
            fee=fee,
        )

        print('Sent {}, with a transaction fee of {}'.format(
            _format_mob(pmob2mob(transaction_log['value_pmob'])),
            _format_mob(pmob2mob(transaction_log['fee_pmob'])),
        ))

    def submit(self, proposal, account_id=None, receipt=False):
        if account_id is not None:
            account = self._load_account_prefix(account_id)
            account_id = account['account_id']

        with Path(proposal).open() as f:
            tx_proposal = json.load(f)

        # Check that the tombstone block is within range.
        tombstone_block = int(tx_proposal['tx']['prefix']['tombstone_block'])
        network_status = self.client.get_network_status()
        lo = int(network_status['network_block_height']) + 1
        hi = lo + MAX_TOMBSTONE_BLOCKS - 1
        if lo >= tombstone_block:
            print('This transaction has expired, and can no longer be submitted.')
            return
        if tombstone_block > hi:
            print('This transaction cannot be submitted yet. Wait for {} more blocks.'.format(
                tombstone_block - hi))

        # Generate a receipt for the transaction.
        if receipt:
            receipt = self.client.create_receiver_receipts(tx_proposal)
            path = Path('receipt.json')
            if path.exists():
                print(f'The file {path} already exists. Please rename the existing file and retry.')
                return
            else:
                with path.open('w') as f:
                    json.dump(receipt, f, indent=2)
                print(f'Wrote {path}')

        # Confirm and submit.
        if account_id is None:
            print('This transaction will not be logged, because an account id was not provided.')
        total_value = sum( pmob2mob(outlay['value']) for outlay in tx_proposal['outlay_list'] )
        if not self.confirm(
            'Submit this transaction proposal for {}? (Y/N) '.format(_format_mob(total_value))
        ):
            print('Cancelled.')
            return

        self.client.submit_transaction(tx_proposal)
        print('Submitted. The file {} is now unusable for sending transactions.'.format(proposal))

    def qr(self, account_id):
        try:
            import segno
        except ImportError:
            print('Showing QR codes requires the segno library. Try:')
            print('$ pip install segno')
            return

        account = self._load_account_prefix(account_id)
        account_id = account['account_id']

        mob_url = 'mob:///b58/{}'.format(account['main_address'])
        qr = segno.make(mob_url)
        try:
            qr.terminal(compact=True)
        except TypeError:
            qr.terminal()

        print()
        _print_account(account)
        print()

    def address(self, action, **args):
        try:
            getattr(self, 'address_' + action)(**args)
        except TypeError:
            self.address_args.print_help()

    def address_list(self, account_id):
        account = self._load_account_prefix(account_id)
        addresses = self.client.get_addresses_for_account(account['account_id'])

        print()
        print(_format_account_header(account))

        for address in addresses.values():
            print(indent(
                '{} {}'.format(address['public_address'], address['metadata']),
                ' '*2,
            ))
            balance = self.client.get_balance_for_address(address['public_address'])
            print(indent(
                _format_balance(balance),
                ' '*4,
            ))

        print()

    def address_create(self, account_id, metadata):
        account = self._load_account_prefix(account_id)
        address = self.client.assign_address_for_account(account['account_id'], metadata)
        print()
        print(_format_account_header(account))
        print(indent(
            '{} {}'.format(address['public_address'], address['metadata']),
            ' '*2,
        ))
        print()

    def gift(self, action, **args):
        getattr(self, 'gift_' + action)(**args)

    def gift_list(self):
        gift_codes = self.client.get_all_gift_codes()
        if gift_codes == []:
            print('No gift codes.')
        else:
            for gift_code in gift_codes:
                response = self.client.check_gift_code_status(gift_code['gift_code_b58'])
                print()
                _print_gift_code(
                    gift_code['gift_code_b58'],
                    pmob2mob(gift_code['value_pmob']),
                    gift_code['memo'],
                    response['gift_code_status'],
                )
            print()

    def gift_create(self, account_id, amount, memo=''):
        account = self._load_account_prefix(account_id)
        amount = Decimal(amount)
        response = self.client.build_gift_code(account['account_id'], amount, memo)
        gift_code_b58 = response['gift_code_b58']
        tx_proposal = response['tx_proposal']

        print()
        _print_gift_code(gift_code_b58, amount, memo)
        print()
        if not self.confirm(
            'Send {} into this new gift code? (Y/N) '.format(
                _format_mob(amount),
            )
        ):
            print('Cancelled.')
            return

        gift_code = self.client.submit_gift_code(gift_code_b58, tx_proposal, account['account_id'])
        print('Created gift code {}'.format(gift_code['gift_code_b58']))

    def gift_claim(self, account_id, gift_code):
        account = self._load_account_prefix(account_id)
        response = self.client.check_gift_code_status(gift_code)
        amount = pmob2mob(response['gift_code_value'])
        status = response['gift_code_status']
        memo = response.get('gift_code_memo', '')

        if status == 'GiftCodeClaimed':
            print('This gift code has already been claimed.')
            return

        print()
        _print_gift_code(gift_code, amount, memo, status)
        print()
        _print_account(account)
        print()

        if not self.confirm('Claim this gift code for this account? (Y/N) '):
            print('Cancelled.')
            return

        try:
            self.client.claim_gift_code(account['account_id'], gift_code)
        except WalletAPIError as e:
            if e.response['data']['server_error'] == 'GiftCodeClaimed':
                print('This gift code has already been claimed.')
                return

        print('Successfully claimed!')

    def gift_remove(self, gift_code):
        gift_code_b58 = gift_code

        try:
            gift_code = self.client.get_gift_code(gift_code_b58)
            response = self.client.check_gift_code_status(gift_code_b58)

            amount = pmob2mob(response['gift_code_value'])
            status = response['gift_code_status']
            memo = response.get('gift_code_memo', '')
            print()
            _print_gift_code(gift_code_b58, amount, memo, status)
            print()

            if status == 'GiftCodeAvailable':
                print('\n'.join([
                    'This gift code is still available to be claimed.',
                    'If you remove it and lose the code, then the funds in the gift code will be lost.',
                ]))
                if not self.confirm('Continue? (Y/N) '):
                    print('Cancelled.')
                    return

            removed = self.client.remove_gift_code(gift_code_b58)
            assert removed is True
            print('Removed gift code {}'.format(gift_code_b58))

        except WalletAPIError as e:
            if 'GiftCodeNotFound' in e.response['data']['server_error']:
                print('Gift code not found; nothing to remove.')
                return


def _format_mob(mob):
    return '{} MOB'.format(_format_decimal(mob))


def _format_decimal(d):
    # Adapted from https://stackoverflow.com/questions/11227620/drop-trailing-zeros-from-decimal
    d = Decimal(d)
    normalized = d.normalize()
    sign, digit, exponent = normalized.as_tuple()
    result = normalized if exponent <= 0 else normalized.quantize(1)
    return '{:f}'.format(result)


def _format_account_header(account):
    return '{} {}'.format(account['account_id'][:6], account['name'])


def _format_balance(balance):
    offline = False
    network_block = int(balance['network_block_height'])
    if network_block == 0:
        offline = True
        network_block = int(balance['local_block_height'])

    orphaned = pmob2mob(balance['orphaned_pmob'])
    if orphaned > 0:
        orphaned_status = ', {} orphaned'.format(_format_mob(orphaned))
    else:
        orphaned_status = ''

    account_block = int(balance['account_block_height'])
    if account_block == network_block:
        sync_status = 'synced'
    else:
        sync_status = 'syncing, {}/{}'.format(balance['account_block_height'], network_block)

    if offline:
        offline_status = ' [offline]'
    else:
        offline_status = ''

    result = '{}{} ({}){}'.format(
        _format_mob(pmob2mob(balance['unspent_pmob'])),
        orphaned_status,
        sync_status,
        offline_status,
    )
    return result


def _format_gift_code_status(status):
    return {
        'GiftCodeSubmittedPending': 'pending',
        'GiftCodeAvailable': 'available',
        'GiftCodeClaimed': 'claimed',
    }[status]


def _print_account(account, balance=None):
    print(_format_account_header(account))
    print(indent(
        'address {}'.format(account['main_address']),
        ' '*2,
    ))
    if balance is not None:
        print(indent(
            _format_balance(balance),
            ' '*2,
        ))


def _print_gift_code(gift_code_b58, amount, memo='', status=None):
    lines = []
    lines.append(_format_mob(amount))
    if memo:
        lines.append(memo)
    if status is not None:
        lines.append('({})'.format(_format_gift_code_status(status)))
    print(gift_code_b58)
    print(indent('\n'.join(lines), ' '*2))


def _print_txo(txo, received=False):
    print(txo)
    to_address = txo['assigned_address']
    if received:
        verb = 'Received'
    else:
        verb = 'Spent'
    print('  {} {}'.format(verb, _format_mob(pmob2mob(txo['value_pmob']))))
    if received:
        if int(txo['subaddress_index']) == 1:
            print('    as change')
        else:
            print('    at subaddress {}, {}'.format(
                txo['subaddress_index'],
                to_address,
            ))
    else:
        print('    to unknown address')


def _load_import(backup):
    # Try to load it as a file.
    try:
        return _load_import_file(backup)
    except FileNotFoundError:
        pass

    # Try to use the legacy import system, treating the string as hexadecimal root entropy.
    try:
        b = bytes.fromhex(backup)
        if len(b) == 32:
            return {'legacy_root_entropy': b.hex()}
    except ValueError:
        pass

    # Lastly, assume that this is just a mnemonic phrase written to the command line.
    return {'mnemonic': backup}


def _load_import_file(filename):
    result = {}

    with open(filename) as f:
        data = json.load(f)

    for field in [
        'mnemonic',  # Key derivation version 2+.
        'key_derivation_version',
        'legacy_root_entropy',  # Key derivation version 1.
        'name',
        'first_block_index',
        'next_subaddress_index',
    ]:
        value = data.get(field)
        if value is not None:
            result[field] = value

    result['fog_keys'] = {}
    for field in [
        'fog_report_url',
        'fog_report_id',
        'fog_authority_spki',
    ]:
        value = data['account_key'].get(field)
        if value is not None:
            result['fog_keys'][field] = value

    return result


def _save_export(account, secrets, filename):
    export_data = {}

    mnemonic = secrets.get('mnemonic')
    if mnemonic is not None:
        export_data['mnemonic'] = mnemonic
        export_data['key_derivation_version'] = secrets['key_derivation_version']
    legacy_root_entropy = secrets.get('entropy')
    if legacy_root_entropy is not None:
        export_data['root_entropy'] = legacy_root_entropy

    export_data.update({
        'account_id': account['account_id'],
        'account_name': account['name'],
        'account_key': secrets['account_key'],
        'first_block_index': account['first_block_index'],
        'next_subaddress_index': account['next_subaddress_index'],
    })

    path = Path(filename)
    if path.exists():
        raise OSError('File exists.')
    with path.open('w') as f:
        json.dump(export_data, f, indent=2)
        f.write('\n')
