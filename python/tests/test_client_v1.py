import os
import json
import re
import pytest

from mobilecoin.client_v1 import (
    Client,
    WalletAPIError,
)
from mobilecoin.token import get_token, Amount

MOB = get_token('MOB')

@pytest.fixture(scope='session')
def client():
    yield Client(verbose=True)


@pytest.fixture(scope='session')
def fee(client):
    """Get the default fee amount."""
    network_status = client.get_network_status()
    return Amount.from_storage_units(network_status['fee_pmob'], MOB)


@pytest.fixture(scope='session')
def source_account(client):
    """
    Import the account specified by the MC_WALLET_FILE environment variable,
    and return its account id.
    """
    wallet_file = os.environ['MC_WALLET_FILE']
    with open(wallet_file) as f:
        data = json.load(f)

    account_id = None
    try:
        # Import an account.
        account = client.import_account(
            mnemonic=data['mnemonic'],
            first_block_index=data['first_block_index'],
        )
        account_id = account['account_id']

    except WalletAPIError as e:
        # If we have already imported this account, get the account id from the
        # server error text with a regex, and load the account.
        error_text = e.response['error']['data']['server_error']
        match = re.search(r'AccountAlreadyExists\("(.*?)"\)', error_text)
        if match is not None:
            account_id = match.group(1)
        else:
            raise

    assert account_id is not None

    # Wait for the account to sync, and check that it has sufficient balance.
    balance = client.poll_balance(account_id, timeout=60)
    initial_balance = Amount.from_storage_units(
        balance['unspent_pmob'],
        MOB,
    )
    assert initial_balance >= Amount.from_display_units(0.1, MOB)

    return client.get_account(account_id)


@pytest.fixture
def account_factory(client, source_account, fee):
    class AccountFactory:
        def __init__(self):
            self.temp_accounts = []

        def create(self):
            temp_account = client.create_account()
            self.temp_accounts.append(temp_account)
            return temp_account

        def create_fog(self):
            temp_fog_account = client.create_account(
                fog_report_url=os.environ['MC_FOG_REPORT_URL'],
                fog_authority_spki=os.environ['MC_FOG_AUTHORITY_SPKI'],
            )
            self.temp_accounts.append(temp_fog_account)
            return temp_fog_account

        def cleanup(self):
            for temp_account in self.temp_accounts:
                _clean_up_temp_account(client, source_account, temp_account, fee)

    account_factory = AccountFactory()

    yield account_factory

    account_factory.cleanup()


def _clean_up_temp_account(client, source_account, temp_account, fee):
    # Send all funds back to the main account.
    tx_index = None
    balance = client.get_balance_for_account(temp_account['account_id'])
    amount = Amount.from_storage_units(balance['unspent_pmob'], MOB)
    if amount >= fee:
        send_amount = amount - fee
        transaction_log, _ = client.build_and_submit_transaction(
            temp_account['account_id'],
            send_amount,
            source_account['main_address'],
        )
        tx_index = int(transaction_log['submitted_block_index'])

    # Ensure the temporary account has no funds remaining.
    if tx_index is not None:
        balance = client.poll_balance(temp_account['account_id'], tx_index + 1)
        assert int(balance['unspent_pmob']) == 0

    # Delete the temporary account.
    client.remove_account(temp_account['account_id'])


def test_version(client):
    version = client.version()
    assert isinstance(version, dict)
    assert sorted(version.keys()) == ['commit', 'number', 'string']


def test_network_status(client):
    network_status = client.get_network_status()

    assert sorted(network_status.keys()) == [
        'block_version',
        'fee_pmob',
        'local_block_height',
        'network_block_height',
        'object',
    ]


def test_network_status_fee(fee):
    assert fee.token.short_code == 'MOB'


def test_get_block(client):
    network_status = client.get_network_status()
    last_block_index = int(network_status['local_block_height']) - 1
    block, block_contents = client.get_block(last_block_index)
    assert int(block['index']) == last_block_index
    assert sorted(block.keys()) == [
        'contents_hash',
        'cumulative_txo_count',
        'id',
        'index',
        'parent_id',
        'root_element',
        'version',
    ]
    assert sorted(block_contents.keys()) == ['key_images', 'outputs']


def test_wallet_status(client):
    wallet_status = client.get_wallet_status()
    assert sorted(wallet_status.keys()) == [
        'account_ids',
        'account_map',
        'is_synced_all',
        'local_block_height',
        'min_synced_block_index',
        'network_block_height',
        'object',
        'total_orphaned_pmob',
        'total_pending_pmob',
        'total_secreted_pmob',
        'total_spent_pmob',
        'total_unspent_pmob'
    ]


def test_get_balance_for_account(client, source_account):
    balance = client.get_balance_for_account(source_account['account_id'])
    assert sorted(balance.keys()) == [
        'account_block_height',
        'is_synced',
        'local_block_height',
        'max_spendable_pmob',
        'network_block_height',
        'object',
        'orphaned_pmob',
        'pending_pmob',
        'secreted_pmob',
        'spent_pmob',
        'unspent_pmob',
    ]


def test_send_transaction_self(client, source_account, fee):
    # Check the initial balance.
    balance = client.get_balance_for_account(source_account['account_id'])
    initial_balance = Amount.from_storage_units(balance['unspent_pmob'], MOB)

    # Send a transaction from the account back to itself.
    transaction_log, _ = client.build_and_submit_transaction(
        source_account['account_id'],
        Amount.from_display_units(0.01, MOB),
        source_account['main_address'],
    )
    tx_value = Amount.from_storage_units(transaction_log['value_pmob'], MOB)
    assert tx_value == Amount.from_display_units(0.01, MOB)

    # Wait for the account to sync.
    tx_index = int(transaction_log['submitted_block_index'])
    balance = client.poll_balance(source_account['account_id'], tx_index + 1)

    # Check that the balance has decreased by the fee amount.
    final_balance = Amount.from_storage_units(balance['unspent_pmob'], MOB)
    assert final_balance == initial_balance - fee


def _test_send_transaction(client, account, temp_account):
    # Send a transaction to the temporary account.
    transaction_log, _ = client.build_and_submit_transaction(
        account['account_id'],
        Amount.from_display_units(0.01, MOB),
        temp_account['main_address'],
    )
    tx_value = Amount.from_storage_units(transaction_log['value_pmob'], MOB)
    assert tx_value == Amount.from_display_units(0.01, MOB)

    # Wait for the temporary account to sync.
    tx_index = int(transaction_log['submitted_block_index'])
    temp_balance = client.poll_balance(temp_account['account_id'], tx_index + 1)

    # Check that the transaction has arrived.
    temp_balance = Amount.from_storage_units(
        temp_balance['unspent_pmob'],
        MOB,
    )
    assert temp_balance == Amount.from_display_units(0.01, MOB)


def test_send_transaction(client, source_account, account_factory):
    temp_account = account_factory.create()
    _test_send_transaction(client, source_account, temp_account)


@pytest.mark.skipif(
    (
        'MC_FOG_REPORT_URL' not in os.environ or
        'MC_FOG_AUTHORITY_SPKI' not in os.environ
    ),
    reason='Fog server not given.'
)
def test_send_transaction_fog(client, source_account, account_factory):
    temp_fog_account = account_factory.create_fog()
    _test_send_transaction(client, source_account, temp_fog_account)


def test_send_transaction_subaddress(client, source_account, account_factory):
    temp_account = account_factory.create()

    # Create a new address for the temporary account.
    address = client.assign_address_for_account(temp_account['account_id'])
    address = address['public_address']

    # Send a transaction to the temporary account.
    transaction_log, _ = client.build_and_submit_transaction(
        source_account['account_id'],
        Amount.from_display_units(0.01, MOB),
        address,
    )

    # Wait for the temporary account to sync.
    tx_index = int(transaction_log['submitted_block_index'])
    temp_balance = client.poll_balance(temp_account['account_id'], tx_index + 1)

    # Check that the transaction has arrived.
    temp_balance = Amount.from_storage_units(temp_balance['unspent_pmob'], MOB)
    assert temp_balance == Amount.from_display_units(0.01, MOB)

    # The subaddress balance also shows the transaction.
    balance = client.get_balance_for_address(address)
    subaddress_balance = Amount.from_storage_units(balance['unspent_pmob'], MOB)
    assert subaddress_balance == Amount.from_display_units(0.01, MOB)


def test_build_transaction_multiple_outputs(client, source_account, account_factory):
    temp_account_1 = account_factory.create()
    temp_account_2 = account_factory.create()

    tx_proposal, _ = client.build_transaction(
        source_account['account_id'],
        {
            temp_account_1['main_address']: Amount.from_display_units(0.01, MOB),
            temp_account_2['main_address']: Amount.from_display_units(0.01, MOB),
        },
    )
    transaction_log = client.submit_transaction(
        tx_proposal,
        source_account['account_id'],
    )

    # Wait for the temporary accounts to sync.
    tx_index = int(transaction_log['submitted_block_index'])
    balances = [
        client.poll_balance(temp_account_1['account_id'], tx_index + 1),
        client.poll_balance(temp_account_2['account_id'], tx_index + 1),
    ]

    # Check that the transactions have arrived.
    for balance in balances:
        balance = Amount.from_storage_units(
            balance['unspent_pmob'],
            MOB,
        )
        assert balance == Amount.from_display_units(0.01, MOB)
