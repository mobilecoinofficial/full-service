import os
import pytest

from mobilecoin.client_v1 import Client, WalletAPIError
from mobilecoin.token import get_token, Amount

MOB = get_token('MOB')


@pytest.fixture(scope='session')
def client_v1():
    yield Client(verbose=True)


@pytest.fixture(scope='session')
def fee(client_v1):
    """Get the default fee amount."""
    network_status = client_v1.get_network_status()
    return Amount.from_storage_units(network_status['fee_pmob'], MOB)


def test_version(client_v1):
    version = client_v1.version()
    assert isinstance(version, dict)
    assert sorted(version.keys()) == ['commit', 'number', 'string']


def test_network_status(client_v1):
    network_status = client_v1.get_network_status()

    assert sorted(network_status.keys()) == [
        'block_version',
        'fee_pmob',
        'local_block_height',
        'network_block_height',
        'object',
    ]


def test_network_status_fee(fee):
    assert fee.token.short_code == 'MOB'


def test_get_block(client_v1):
    network_status = client_v1.get_network_status()
    last_block_index = int(network_status['local_block_height']) - 1
    block, block_contents = client_v1.get_block(last_block_index)
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


def test_wallet_status(client_v1):
    wallet_status = client_v1.get_wallet_status()
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


def test_get_balance_for_account(client_v1, source_account):
    balance = client_v1.get_balance_for_account(source_account['id'])
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


def test_create_account(client_v1):
    account = client_v1.create_account(name='test_account')
    assert account['name'] == 'test_account'
    client_v1.remove_account(account['account_id'])
    with pytest.raises(WalletAPIError):
        client_v1.get_account(account['account_id'])


def test_send_transaction_self(client_v1, source_account, fee):
    # Check the initial balance.
    balance = client_v1.get_balance_for_account(source_account['id'])
    initial_balance = Amount.from_storage_units(balance['unspent_pmob'], MOB)

    # Send a transaction from the account back to itself.
    transaction_log, _ = client_v1.build_and_submit_transaction(
        source_account['id'],
        Amount.from_display_units(0.001, MOB),
        source_account['main_address'],
    )
    tx_value = Amount.from_storage_units(transaction_log['value_pmob'], MOB)
    assert tx_value == Amount.from_display_units(0.001, MOB)

    # Wait for the account to sync.
    tx_index = int(transaction_log['submitted_block_index'])
    balance = client_v1.poll_balance(source_account['id'], tx_index + 1)

    # Check that the balance has decreased by the fee amount.
    final_balance = Amount.from_storage_units(balance['unspent_pmob'], MOB)
    assert final_balance == initial_balance - fee


def _test_send_transaction(client_v1, account, temp_account):
    # Send a transaction to the temporary account.
    transaction_log, _ = client_v1.build_and_submit_transaction(
        account['id'],
        Amount.from_display_units(0.001, MOB),
        temp_account['main_address'],
    )
    tx_value = Amount.from_storage_units(transaction_log['value_pmob'], MOB)
    assert tx_value == Amount.from_display_units(0.001, MOB)

    # Wait for the temporary account to sync.
    tx_index = int(transaction_log['submitted_block_index'])
    temp_balance = client_v1.poll_balance(temp_account['id'], tx_index + 1)

    # Check that the transaction has arrived.
    temp_balance = Amount.from_storage_units(
        temp_balance['unspent_pmob'],
        MOB,
    )
    assert temp_balance == Amount.from_display_units(0.001, MOB)


async def test_send_transaction(client_v1, source_account, account_factory):
    temp_account = await account_factory.create()
    _test_send_transaction(client_v1, source_account, temp_account)


@pytest.mark.skipif(
    (
        'MC_FOG_REPORT_URL' not in os.environ or
        'MC_FOG_AUTHORITY_SPKI' not in os.environ
    ),
    reason='Fog server not given.'
)
async def test_send_transaction_fog(client_v1, source_account, account_factory):
    temp_fog_account = await account_factory.create_fog()
    _test_send_transaction(client_v1, source_account, temp_fog_account)

# failed - not waiting for primary account to be synced
async def test_send_transaction_subaddress(client_v1, source_account, account_factory):
    temp_account = await account_factory.create()

    # Create a new address for the temporary account.
    address = client_v1.assign_address_for_account(temp_account['id'])
    address = address['public_address']

    # Start with zero balance in the subaddress.
    balance = client_v1.get_balance_for_address(address)
    subaddress_balance = Amount.from_storage_units(balance['unspent_pmob'], MOB)
    assert subaddress_balance == Amount.from_display_units(0, MOB)

    # Send a transaction to the temporary account.
    transaction_log, _ = client_v1.build_and_submit_transaction(
        source_account['id'],
        Amount.from_display_units(0.001, MOB),
        address,
    )

    # Wait for the temporary account to sync.
    tx_index = int(transaction_log['submitted_block_index'])
    temp_balance = client_v1.poll_balance(temp_account['id'], tx_index + 1)

    # Check that the transaction has arrived.
    temp_balance = Amount.from_storage_units(temp_balance['unspent_pmob'], MOB)
    assert temp_balance == Amount.from_display_units(0.001, MOB)

    # The subaddress balance also shows the transaction.
    balance = client_v1.get_balance_for_address(address)
    subaddress_balance = Amount.from_storage_units(balance['unspent_pmob'], MOB)
    assert subaddress_balance == Amount.from_display_units(0.001, MOB)

async def test_build_transaction_multiple_outputs(client_v1, source_account, account_factory):
    temp_account_1 = await account_factory.create()
    temp_account_2 = await account_factory.create()

    tx_proposal, _ = client_v1.build_transaction(
        source_account['id'],
        {
            temp_account_1['main_address']: Amount.from_display_units(0.001, MOB),
            temp_account_2['main_address']: Amount.from_display_units(0.001, MOB),
        },
    )
    transaction_log = client_v1.submit_transaction(
        tx_proposal,
        source_account['id'],
    )

    # Wait for the temporary accounts to sync.
    tx_index = int(transaction_log['submitted_block_index'])
    balances = [
        client_v1.poll_balance(temp_account_1['id'], tx_index + 1),
        client_v1.poll_balance(temp_account_2['id'], tx_index + 1),
    ]

    # Check that the transactions have arrived.
    for balance in balances:
        balance = Amount.from_storage_units(
            balance['unspent_pmob'],
            MOB,
        )
        assert balance == Amount.from_display_units(0.001, MOB)
