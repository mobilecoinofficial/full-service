from contextlib import contextmanager
from decimal import Decimal
import sys
import tempfile
import time

from mobilecoin import (
    Client,
    WalletAPIError,
    pmob2mob,
)
from mobilecoin.cli import (
    CommandLineInterface,
    _load_import,
)


def main():
    c = Client(verbose=False)

    source_wallet = sys.argv[1]

    # Create a test wallet database, and start the server.
    db_file = tempfile.NamedTemporaryFile(suffix='.db', prefix='test_wallet_', delete=False)
    cli = CommandLineInterface()
    cli.config['wallet-db'] = db_file.name
    cli.stop()
    time.sleep(0.5)  # Wait for other servers to stop.
    cli.start(bg=True, unencrypted=True)
    time.sleep(1.5)  # Wait for the server to start listening.

    # Start and end with an empty wallet.
    try:
        check_wallet_empty(c)

        test_errors(c)
        test_account_management(c)
        tests_with_wallet(c, source_wallet)

        check_wallet_empty(c)
    except Exception:
        print('FAIL')
        raise
    else:
        print('ALL PASS')
        cli.stop()  # Only stop the server if there were no errors.


def test_errors(c):
    print('\ntest_errors')

    try:
        c.get_account('invalid')
    except WalletAPIError:
        pass
    else:
        raise AssertionError()

    print('PASS')


def test_account_management(c):
    print('\ntest_account_management')

    # Create an account.
    account = c.create_account()
    account_id = account['account_id']

    # Get accounts.
    account_2 = c.get_account(account_id)
    assert account == account_2

    accounts = c.get_all_accounts()
    account_ids = list(accounts.keys())
    assert account_ids == [account_id]
    assert accounts[account_id] == account

    # Rename account.
    assert account['name'] == ''
    c.update_account_name(account_id, 'X')
    account = c.get_account(account_id)
    assert account['name'] == 'X'

    # Remove the created account.
    c.remove_account(account_id)

    # Import an account from entropy.
    entropy = '0000000000000000000000000000000000000000000000000000000000000000'
    account = c.import_account_from_legacy_root_entropy(entropy)
    account_id = account['account_id']
    assert (
        account['main_address']
        == '6UEtkm1rieLhuz2wvELPHdGiCb96zNnW856QVeGLvYzE7NhmbG1MxnoSPGqyVfEHDvxzQmaURFpZcxT9TSypVgRVAusr7svtD1TcrYj92Uh'
    )

    # Export secrets.
    secrets = c.export_account_secrets(account_id)
    assert secrets['entropy'] == entropy
    assert (
        secrets['account_key']['view_private_key']
        == '0a20b0146de8cd8f5b7962f9e74a5ef0f3e58a9550c9527ac144f38729f0fd3fed0e'
    )
    assert (
        secrets['account_key']['spend_private_key']
        == '0a20b4bf01a77ed4e065e9082d4bda67add30c88e021dcf81fc84e6a9ca2cb68e107'
    )
    c.remove_account(account_id)

    print('PASS')


def tests_with_wallet(c, source_wallet):
    print('\nLoading source wallet', source_wallet)

    # Import an account with money.
    data = _load_import(source_wallet)
    source_account = c.import_account(**data)
    source_account_id = source_account['account_id']

    # Check its balance and make sure it has txos.
    balance = c.poll_balance(source_account_id)
    assert pmob2mob(balance['unspent_pmob']) >= 1
    txos = c.get_all_txos_for_account(source_account_id)
    assert len(txos) > 0

    try:
        test_transaction(c, source_account_id)
        test_prepared_transaction(c, source_account_id)
        test_subaddresses(c, source_account_id)
        test_gift_codes(c, source_account_id)
    except Exception:
        raise
    else:
        c.remove_account(source_account['account_id'])


def test_transaction(c, source_account_id):
    print('\ntest_transaction')

    source_account = c.get_account(source_account_id)

    # Create a temporary account to transact with.
    dest_account = c.create_account()
    dest_account_id = dest_account['account_id']

    # Send transactions and ensure they show up in the transaction list.
    transaction_log = c.build_and_submit_transaction(source_account_id, 0.1, dest_account['main_address'])
    tx_index = int(transaction_log['submitted_block_index'])
    balance = c.poll_balance(dest_account_id, tx_index + 1)
    assert pmob2mob(balance['unspent_pmob']) == Decimal('0.1')

    # Send back the remaining money.
    transaction_log = c.build_and_submit_transaction(dest_account_id, 0.0996, source_account['main_address'])
    tx_index = int(transaction_log['submitted_block_index'])
    balance = c.poll_balance(dest_account_id, tx_index + 1)
    assert pmob2mob(balance['unspent_pmob']) == Decimal('0.0')

    # Check transaction logs.
    transaction_log_map = c.get_all_transaction_logs_for_account(dest_account_id)
    amounts = [ pmob2mob(t['value_pmob']) for t in transaction_log_map.values() ]
    assert sorted( float(a) for a in amounts ) == [0.0996, 0.1], str(amounts)
    assert all( t['status'] == 'tx_status_succeeded' for t in transaction_log_map.values() )

    c.remove_account(dest_account_id)

    print('PASS')


def test_prepared_transaction(c, source_account_id):
    print('\ntest_prepared_transaction')

    source_account = c.get_account(source_account_id)

    # Create a temporary account.
    dest_account = c.create_account()
    dest_account_id = dest_account['account_id']

    # Send a prepared transaction with a receipt.
    tx_proposal = c.build_transaction(source_account_id, 0.1, dest_account['main_address'])
    assert len(tx_proposal['outlay_list']) == 1
    receipts = c.create_receiver_receipts(tx_proposal)
    assert len(receipts) == 1
    receipt = receipts[0]

    status = c.check_receiver_receipt_status(dest_account['main_address'], receipt)
    assert status['receipt_transaction_status'] == 'TransactionPending'

    transaction_log = c.submit_transaction(tx_proposal, source_account_id)
    tx_index = int(transaction_log['submitted_block_index'])

    balance = c.poll_balance(dest_account_id, tx_index + 1)
    assert pmob2mob(balance['unspent_pmob']) == Decimal('0.1')

    status = c.check_receiver_receipt_status(dest_account['main_address'], receipt)
    assert status['receipt_transaction_status'] == 'TransactionSuccess'

    # Send back the remaining money.
    transaction_log = c.build_and_submit_transaction(dest_account_id, 0.0996, source_account['main_address'])
    tx_index = int(transaction_log['submitted_block_index'])
    balance = c.poll_balance(dest_account_id, tx_index + 1)
    assert pmob2mob(balance['unspent_pmob']) == Decimal('0.0')

    c.remove_account(dest_account_id)

    print('PASS')


def test_subaddresses(c, source_account_id):
    print('\ntest_subaddresses')

    addresses = c.get_addresses_for_account(source_account_id)
    source_address = list(addresses.keys())[0]

    # Create a temporary account.
    dest_account = c.create_account()
    dest_account_id = dest_account['account_id']

    # Create a subaddress for the destination account.
    addresses = c.get_addresses_for_account(dest_account_id)
    assert len(addresses) == 2  # Main address and change address.

    address = c.assign_address_for_account(dest_account_id, 'Address Name')
    dest_address = address['public_address']

    addresses = c.get_addresses_for_account(dest_account_id)
    assert len(addresses) == 3
    assert addresses[dest_address]['metadata'] == 'Address Name'

    # Send the subaddress some money.
    transaction_log = c.build_and_submit_transaction(source_account_id, 0.1, dest_address)
    tx_index = int(transaction_log['submitted_block_index'])
    balance = c.poll_balance(dest_account_id, tx_index + 1)
    assert pmob2mob(balance['unspent_pmob']) == Decimal('0.1')

    # The second address has money credited to it, but the main one doesn't.
    balance = c.get_balance_for_address(dest_address)
    assert pmob2mob(balance['unspent_pmob']) == Decimal('0.1')
    balance = c.get_balance_for_address(dest_account['main_address'])
    assert pmob2mob(balance['unspent_pmob']) == Decimal('0.0')

    # Send the money back.
    transaction_log = c.build_and_submit_transaction(dest_account_id, 0.0996, source_address)
    tx_index = int(transaction_log['submitted_block_index'])
    balance = c.poll_balance(dest_account_id, tx_index + 1)
    assert pmob2mob(balance['unspent_pmob']) == Decimal('0.0')

    # The per-address balances account for sent funds.
    balance = c.get_balance_for_address(dest_account['main_address'])
    assert pmob2mob(balance['unspent_pmob']) == Decimal('0.0')
    balance = c.get_balance_for_address(dest_address)
    assert pmob2mob(balance['unspent_pmob']) == Decimal('0.0')

    c.remove_account(dest_account_id)

    print('PASS')


def test_gift_codes(c, source_account_id):
    print('\ntest_gift_codes')

    source_account = c.get_account(source_account_id)

    # Create a gift code.
    response = c.build_gift_code(source_account_id, 0.1, 'abc')
    gift_code_b58 = response['gift_code_b58']
    tx_proposal = response['tx_proposal']
    c.submit_gift_code(gift_code_b58, tx_proposal, source_account_id)

    # Make sure the gift code was funded correctly.
    response = c.poll_gift_code_status(gift_code_b58, 'GiftCodeAvailable')
    assert pmob2mob(response['gift_code_value']) == Decimal('0.1')

    # Create a temporary account.
    dest_account = c.create_account()
    dest_account_id = dest_account['account_id']

    # Claim the gift code.
    # Claimed means the txo was sent, not that it arrived. Poll for the Txo to land.
    txo_id_hex = c.claim_gift_code(dest_account_id, gift_code_b58)
    c.poll_txo(txo_id_hex)
    balance = c.get_balance_for_account(dest_account_id)
    assert pmob2mob(balance['unspent_pmob']) == Decimal('0.0996')

    # Send back the remaining money. We incurred two fees to submit and claim the gift code.
    transaction_log = c.build_and_submit_transaction(dest_account_id, 0.0992, source_account['main_address'])
    tx_index = int(transaction_log['submitted_block_index'])
    balance = c.poll_balance(dest_account_id, tx_index + 1)
    assert pmob2mob(balance['unspent_pmob']) == Decimal('0.0')

    c.remove_account(dest_account_id)

    print('PASS')


def check_wallet_empty(c):
    with quiet(c):
        accounts = c.get_all_accounts()
        assert accounts == {}, 'Wallet not empty!'


@contextmanager
def quiet(c):
    old_verbose = c.verbose
    c.verbose = False
    yield
    c.verbose = old_verbose


if __name__ == '__main__':
    main()
