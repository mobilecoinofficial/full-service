import argparse
from decimal import Decimal
import json
from pathlib import Path
from textwrap import indent

from .client import (
    ClientSync as Client,
    WalletAPIError,
    MAX_TOMBSTONE_BLOCKS,
    log as client_log,
)
from .token import Amount, get_token, TOKENS


def main():
    CommandLineInterface().main()


class CommandLineInterface:

    def main(self):
        self.client = Client()

        # Parse arguments.
        self._create_parsers()
        args = self.parser.parse_args()
        args = vars(args)
        self.auto_confirm = args.pop('yes')
        if args.pop('verbose'):
            client_log.setLevel('DEBUG')

        # Dispatch command.
        command = args.pop('command')
        if command is None:
            self.parser.print_help()
            exit(1)
        setattr(self, 'import', self.import_)  # Can't name a function "import".
        command = command.translate(str.maketrans('-', '_'))
        command_func = getattr(self, command)
        try:
            command_func(**args)
        except ConnectionError as e:
            print(e)
            exit(1)

    def _create_parsers(self):
        self.parser = argparse.ArgumentParser(
            prog='mob',
            description='MobileCoin command-line wallet.',
        )
        self.parser.add_argument('-v', '--verbose', action='store_true', help='Show more information.')
        self.parser.add_argument('-y', '--yes', action='store_true', help='Do not ask for confirmation.')

        command_sp = self.parser.add_subparsers(dest='command', help='Commands')

        # Network status.
        self.status_args = command_sp.add_parser('status', help='Check the status of the MobileCoin network.')

        # List accounts.
        self.list_args = command_sp.add_parser('list', help='List accounts.')

        # Create account.
        self.create_args = command_sp.add_parser('create', help='Create a new account.')
        self.create_args.add_argument('-n', '--name', help='Account name.')
        self.create_args.add_argument('--fog-report-url', help='Fog report server URL.')
        self.create_args.add_argument('--fog-authority-spki',
                                      help='Fog authority subject public key info')

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
        self.send_args.add_argument('amount', help='Amount to send.')
        self.send_args.add_argument('token', help='Token to send (MOB, eUSD).')
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

        # Sync view-only account.
        self.sync_args = command_sp.add_parser('sync', help='Sync a view-only account.')
        self.sync_args.add_argument(
            'account_id_or_sync_response',
            help=(
                'If an account ID is passed, then generate a sync request for the transaction signer. '
                'Once the signer is finished, call this again with the completed json file.'
            )
        )

        # Version
        self.version_args = command_sp.add_parser('version', help='Show version number.')

    def _load_account_prefix(self, prefix):
        accounts = self.client.get_accounts()

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

    def status(self):
        wallet_status = self.client.get_wallet_status()
        network_status = self.client.get_network_status()

        # Show sync state.
        if int(wallet_status['network_block_height']) == 0:
            print('Offline.')
            print('Local ledger has {} blocks.'.format(
                wallet_status['local_block_height']))
        else:
            print('Connected to MobileCoin network.')
            if wallet_status['is_synced_all']:
                print('All accounts synced, {} blocks.'.format(
                    wallet_status['network_block_height']))
            else:
                print('Syncing, {}/{} blocks completed.'.format(
                    wallet_status['min_synced_block_index'],
                    wallet_status['network_block_height'],
                ))

        # Show balances.
        print()
        print('Total balance for all accounts:')
        print(indent(
            _format_balances(wallet_status['balance_per_token']),
            ' '*2,
        ))

        # Show transaction fees.
        print()
        print('Transaction Fees:')
        for token in TOKENS:
            amount = Amount.from_storage_units(
                network_status['fees'][str(token.token_id)],
                token
            )
            print(indent(amount.format(), '  '))

    def list(self):
        accounts = self.client.get_accounts()
        if len(accounts) == 0:
            print('No accounts.')
            return

        account_list = []
        for account_id in accounts.keys():
            status = self.client.get_account_status(account_id)
            account_list.append(status)

        for status in account_list:
            print()
            _print_account(status)

    def create(self, **args):
        account = self.client.create_account(**args)
        print('Created a new account.')
        print(_format_account_header(account))

    def rename(self, account_id, name):
        account = self._load_account_prefix(account_id)
        old_name = account['name']
        account_id = account['id']
        account = self.client.update_account_name(account_id, name)
        print('Renamed account from "{}" to "{}".'.format(
            old_name,
            account['name'],
        ))
        print()
        print(_format_account_header(account))
        print()

    def import_(self, backup, name=None, block=None, key_derivation_version=2):
        account = None
        if backup.endswith('.json'):
            with open(backup) as f:
                data = json.load(f)

            if data.get('method') == 'import_view_only_account':
                account = self.client.import_view_only_account(data['params'])
            else:
                params = {}

                if name is not None:
                    params['name'] = name

                for field in [
                    'mnemonic',
                    'name',
                    'first_block_index',
                    'next_subaddress_index',
                ]:
                    value = data.get(field)
                    if value is not None:
                        params[field] = value

                if 'account_key' in data:
                    for field in [
                        'fog_report_url',
                        'fog_authority_spki',
                    ]:
                        value = data['account_key'].get(field)
                        if value is not None:
                            params[field] = value

                account = self.client.import_account(**params)

        else:
            # Try to use the legacy import system, treating the string as hexadecimal root entropy.
            root_entropy = None
            try:
                b = bytes.fromhex(backup)
                if len(b) == 32:
                    root_entropy = b.hex()
            except ValueError:
                pass
            if root_entropy is not None:
                account = self.client.import_account_from_legacy_root_entropy(root_entropy)
            else:
                # Lastly, assume that this is just a mnemonic phrase written to the command line.
                account = self.client.import_account(backup)

        if account is None:
            print('Could not import account.')
            return

        print('Imported account.')
        print(_format_account_header(account))
        print()

    def export(self, account_id, show=False):
        account = self._load_account_prefix(account_id)
        account_id = account['id']
        status = self.client.get_account_status(account_id)

        print('You are about to export the secret entropy mnemonic for this account:')
        print()
        _print_account(status)

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
            filename = 'mobilecoin_secret_mnemonic_{}.json'.format(account_id[:6])
            try:
                _save_export(account, secrets, filename)
            except OSError as e:
                print('Could not write file: {}'.format(e))
                exit(1)
            else:
                print(f'Wrote {filename}.')

    def remove(self, account_id):
        account = self._load_account_prefix(account_id)
        account_id = account['id']
        status = self.client.get_account_status(account_id)

        if account.get('view_only'):
            print('You are about to remove this view key:')
            print()
            _print_account(status)
            print()
            print('You will lose the ability to see related transactions unless you')
            print('restore it from backup.')
        else:
            print('You are about to remove this account:')
            print()
            _print_account(status)
            print()
            print('You will lose access to this account unless you restore it')
            print('from the mnemonic phrase.')

        if not self.confirm('Continue? (Y/N) '):
            print('Cancelled.')
            return

        self.client.remove_account(account_id)
        print('Removed.')

    def history(self, account_id):
        account = self._load_account_prefix(account_id)
        account_id = account['id']

        own_addresses = self.client.get_addresses(account_id)
        own_addresses = set( a['public_address_b58'] for a in own_addresses.values() )

        transactions = self.client.get_transaction_logs(account_id, limit=1000)

        def tx_block(t):
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

        transactions = sorted(transactions.values(), key=tx_block)
        for t in transactions:
            fee = Amount.from_storage_units(
                t['fee_amount']['value'],
                t['fee_amount']['token_id'],
            )

            print()
            print('Block #{}, {} output{}'.format(
                tx_block(t),
                len(t['output_txos']),
                '' if len(t['output_txos']) == 1 else 's',
            ))
            print('  Fee:', fee.format())

            for i, txo in enumerate(t['output_txos']):
                print('  Output #{}'.format(i+1))
                amount = Amount.from_storage_units(
                    txo['amount']['value'],
                    txo['amount']['token_id'],
                )
                print(indent(amount.format(), '    '))
                address = txo['recipient_public_address_b58'] 
                if address in own_addresses:
                    print('    Received at', address)
                else:
                    print('    Sent to', address)

    def send(self, account_id, amount, token, to_address, build_only=False, fee=None):
        token = get_token(token)

        account = self._load_account_prefix(account_id)
        account_id = account['id']

        account_status = self.client.get_account_status(account_id)
        try:
            unspent = Amount.from_storage_units(
                account_status['balance_per_token'][str(token.token_id)]['unspent'],
                token,
            )
        except KeyError:
            unspent = Amount.from_storage_units(0, token)

        if fee is None:
            network_status = self.client.get_network_status()
            fee = Amount.from_storage_units(
                network_status['fees'][str(token.token_id)],
                token
            )
        else:
            fee = Amount.from_display_units(fee, token)
        assert fee is not None

        if amount == "all":
            amount = unspent - fee
            total_amount = unspent
        else:
            amount = Amount.from_display_units(amount, token)
            total_amount = amount + fee

        if unspent < total_amount:
            print('There is not enough {} in account {} to pay for this transaction.'.format(
                token.short_code,
                account_id[:6],
            ))
            return

        if account.get('view_only'):
            verb = 'Building unsigned transaction for'
        elif build_only:
            verb = 'Building transaction for'
        else:
            verb = 'Sending'

        print('\n'.join([
            '',
            '{} {}',
            '  from account {}',
            '  to address {}',
            'Fee is {}, for a total amount of {}.',
        ]).format(
            verb,
            amount.format(),
            _format_account_header(account),
            to_address,
            fee.format(),
            total_amount.format(),
        ))
        print()

        if build_only:
            tx_proposal = self.client.build_transaction(
                account_id,
                {to_address, amount},
                fee=fee,
            )
            path = Path('tx_proposal.json')
            if path.exists():
                print(f'The file {path} already exists. Please rename the existing file and retry.')
            else:
                with path.open('w') as f:
                    json.dump(tx_proposal, f, indent=2)
                print(f'Wrote {path}.')
            return

        if account.get('view_only'):
            response = self.client.build_unsigned_transaction(
                account_id, 
                {to_address: amount},
                fee=fee,
            )
            path = Path('tx_proposal_{}_{}_unsigned.json'.format(
                account_id[:6],
                account_status['local_block_height'],
            ))
            if path.exists():
                print(f'The file {path} already exists. Please rename the existing file and retry.')
            else:
                _save_json_file(path, response)
                print(f'Wrote {path}.')
                print()
                print('This account is view-only, so its spend key is in an offline signer.')
                print('Run `transaction-signer sign`, then submit the result with `mob submit`')
            return

        if not self.confirm('Confirm? (Y/N) '):
            print('Cancelled.')
            return

        transaction_log, tx_proposal = self.client.build_and_submit_transaction(
            account_id,
            amount,
            to_address,
            fee=fee,
        )

        sent_amounts = [
            Amount.from_storage_units(value, token_id)
            for token_id, value in transaction_log['value_map'].items()
        ]
        fee_amount = Amount.from_storage_units(
            transaction_log['fee_amount']['value'],
            transaction_log['fee_amount']['token_id'],
        )
        print('Sent {}, with a transaction fee of {}'.format(
            ', '.join(a.format() for a in sent_amounts),
            fee_amount.format(),
        ))

    def submit(self, proposal, account_id=None, receipt=False):
        if account_id is not None:
            account = self._load_account_prefix(account_id)
            account_id = account['id']

        with Path(proposal).open() as f:
            tx_proposal = json.load(f)

        # Check whether this is an already built response from the offline transaction signer.
        if tx_proposal.get('method') == 'submit_transaction':
            account_id = tx_proposal['params']['account_id']
            tx_proposal = tx_proposal['params']['tx_proposal']

        # Check that the tombstone block is within range.
        tombstone_block = int(tx_proposal['tombstone_block_index'])
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
                print(f'Wrote {path}.')

        # Confirm and submit.
        if account_id is None:
            print('This transaction will not be logged, because an account id was not provided.')

        total_amounts = {
            token: Amount.from_storage_units(0, token)
            for token in TOKENS
        }
        for txo in tx_proposal['payload_txos']:
            amount = Amount.from_storage_units(
                txo['amount']['value'],
                txo['amount']['token_id'],
            )
            total_amounts[amount.token] += amount

        print('Submitting a transaction for')
        for token in TOKENS:
            amount = total_amounts[token]
            if amount.value != 0:
                print(indent(amount.format(), '  '))

        if not self.confirm(
            'Send this transaction? (Y/N) '
        ):
            print('Cancelled.')
            return

        self.client.submit_transaction(tx_proposal, account_id)
        print('Submitted. The file {} is now unusable for sending transactions.'.format(proposal))

    def qr(self, account_id):
        try:
            import segno
        except ImportError:
            print('Showing QR codes requires the segno library. Try:')
            print('$ pip install segno')
            return

        account = self._load_account_prefix(account_id)
        account_id = account['id']

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
        print()
        print(_format_account_header(account))

        addresses = self.client.get_addresses(account['id'], limit=1000)
        addresses = list(addresses.values())
        addresses.sort(key=lambda a: int(a['subaddress_index']))

        for address in addresses:
            address_status = self.client.get_address_status(address['public_address_b58'])

            print()
            print('#{} {}'.format(
                address['subaddress_index'],
                address['metadata'],
            ))
            print(indent(address['public_address_b58'], '  '))
            print(indent(_format_balances(address_status['balance_per_token']), '  '))

        print()

    def address_create(self, account_id, metadata):
        account = self._load_account_prefix(account_id)
        address = self.client.assign_address_for_account(account['id'], metadata)
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
        response = self.client.build_gift_code(account['id'], amount, memo)
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

        gift_code = self.client.submit_gift_code(gift_code_b58, tx_proposal, account['id'])
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
            self.client.claim_gift_code(account['id'], gift_code)
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

    def sync(self, account_id_or_sync_response):
        if account_id_or_sync_response.endswith('.json'):
            sync_response = account_id_or_sync_response
            self._finish_sync(sync_response)
        else:
            account_id = account_id_or_sync_response
            self._start_sync(account_id)

    def _start_sync(self, account_id):
        account = self._load_account_prefix(account_id)
        print()
        print(_format_account_header(account))

        account_id = account['id']
        response = self.client.create_view_only_account_sync_request(account_id)

        network_status = self.client.get_network_status()
        filename = 'sync_request_{}_{}.json'.format(account_id[:6], network_status['local_block_height'])
        _save_json_file(filename, response)

        print(f'Wrote {filename}.')

    def _finish_sync(self, sync_response):
        with open(sync_response) as f:
            data = json.load(f)

        self.client.sync_view_only_account(data['params'])
        account_id = data['params']['account_id']
        num_synced = len(data['params']['completed_txos'])

        status = self.client.get_account_status(account_id)

        print()
        print('Synced {} transaction output{}.'.format(
            num_synced,
            '' if num_synced == 1 else '',
        ))
        print()
        _print_account(status)

    def version(self):
        version = self.client.version()
        print('MobileCoin full-service', version['string'])
        print('commit', version['commit'][:6])


def _print_account(status):
    print(
        '{} ({})'.format(
            _format_account_header(status['account']),
            _format_sync_state(status),
        )
    )
    print(indent(
        'address {}'.format(status['account']['main_address']),
        ' '*2,
    ))
    print(indent(
        _format_balances(status['balance_per_token']),
        ' '*2,
    ))


def _format_account_header(account):
    output = account['id'][:6]
    if account['name']:
        output += ' ' + account['name']
    if account.get('view_only'):
        output += ' [view-only]'
    return output


def _format_sync_state(status):
    account_block = int(status['account']['next_block_index'])

    offline = False
    network_block = int(status['network_block_height'])
    if network_block == 0:
        offline = True
        network_block = int(status['local_block_height'])

    if account_block == network_block:
        sync_state = 'synced'
    else:
        sync_state = 'syncing, {}/{}'.format(account_block, network_block)

    if offline:
        offline_state = ' [offline]'
    else:
        offline_state = ''

    return '{}{}'.format(sync_state, offline_state)


def _format_balances(balances):
    lines = []
    for token_id, balance in balances.items():
        unspent = Amount.from_storage_units(balance['unspent'], token_id)
        unverified = Amount.from_storage_units(balance['unverified'], token_id)
        if unspent.value > 0:
            lines.append(unspent.format())
        if unverified.value > 0:
            lines.append('{} unverified'.format(unverified.format()))

    if len(lines) == 0:
        return 'Empty'

    return '\n'.join(lines)


def _format_gift_code_status(status):
    return {
        'GiftCodeSubmittedPending': 'pending',
        'GiftCodeAvailable': 'available',
        'GiftCodeClaimed': 'claimed',
    }[status]


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
        'account_id': account['id'],
        'name': account['name'],
        'account_key': secrets['account_key'],
        'first_block_index': account['first_block_index'],
        'next_subaddress_index': account['next_subaddress_index'],
    })

    _save_json_file(filename, export_data)


def _save_view_key_export(account, secrets, filename):
    _save_json_file(
        filename,
        {
            'name': account['name'],
            'view_private_key': secrets['account_key']['view_private_key'],
            'first_block_index': account['first_block_index'],
        }
    )


def _save_json_file(filename, data):
    path = Path(filename)
    if path.exists():
        raise OSError('File exists.')
    with path.open('w') as f:
        json.dump(data, f, indent=2)
        f.write('\n')
