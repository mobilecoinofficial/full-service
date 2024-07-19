import asyncio
import os
import pytest  # Fixtures are in conftest.py

from mobilecoin.client import WalletAPIError
from mobilecoin.token import get_token, Amount

MOB = get_token('MOB')
EUSD = get_token('EUSD')


async def test_version(client):
    version = await client.version()
    assert isinstance(version, dict)
    assert sorted(version.keys()) == ['commit', 'number', 'string']


async def test_network_status(client):
    network_status = await client.get_network_status()
    assert sorted(network_status.keys()) == [
        'block_version',
        'fees',
        'local_block_height',
        'local_num_txos',
        'max_tombstone_blocks',
        'network_block_height',
        'network_info'
    ]


async def test_network_status_fees(fees):
    # filter out FauxUSD as not on mainnet
    fee_tokens = [t.short_code for t in fees.keys() if t.short_code != 'FauxUSD']
    assert sorted(fee_tokens) == ['MOB', 'eUSD']
    assert all(isinstance(a, Amount) for a in fees.values())


async def test_get_block(client):
    network_status = await client.get_network_status()
    last_block_index = int(network_status['local_block_height']) - 1
    block, block_contents = await client.get_block(last_block_index)
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


async def test_wallet_status(client):
    wallet_status = await client.get_wallet_status()
    assert sorted(wallet_status.keys()) == [
        'balance_per_token',
        'is_synced_all',
        'local_block_height',
        'min_synced_block_index',
        'network_block_height',
    ]


async def test_account_status(client, source_account):
    status = await client.get_account_status(source_account['id'])
    assert sorted(status.keys()) == [
        'account',
        'balance_per_token',
        'local_block_height',
        'network_block_height',
    ]
    assert sorted(status['account'].keys()) == [
        'first_block_index',
        'fog_enabled',
        'id',
        'key_derivation_version',
        'main_address',
        'managed_by_hardware_wallet',
        'name',
        'next_block_index',
        'next_subaddress_index',
        'recovery_mode',
        'require_spend_subaddress',
        'view_only',
    ]


async def test_create_account(client):
    account = await client.create_account(name='test_account')
    assert account['name'] == 'test_account'
    await client.remove_account(account['id'])
    with pytest.raises(WalletAPIError):
        await client.get_account_status(account['id'])


async def test_send_transaction_self(client, source_account, fees):
    # Check the initial balance.
    status = await client.get_account_status(source_account['id'])
    initial_balance = Amount.from_storage_units(
        status['balance_per_token'][str(MOB.token_id)]['unspent'],
        MOB,
    )

    # Send a transaction from the account back to itself.
    transaction_log, _ = await client.build_and_submit_transaction(
        source_account['id'],
        Amount.from_display_units(0.01, MOB),
        source_account['main_address'],
    )
    tx_value = Amount.from_storage_units(
        transaction_log['value_map'][str(MOB.token_id)],
        MOB,
    )
    assert tx_value == Amount.from_display_units(0.01, MOB)

    # Wait for the account to sync.
    tx_index = int(transaction_log['submitted_block_index'])
    status = await client.poll_account_status(source_account['id'], tx_index + 1)

    # Check that the balance has decreased by the fee amount.
    final_balance = Amount.from_storage_units(
        status['balance_per_token'][str(MOB.token_id)]['unspent'],
        MOB,
    )
    assert final_balance == initial_balance - fees[MOB]


async def _test_send_transaction(client, account, temp_account):
    # Send a transaction to the temporary account.
    transaction_log, _ = await client.build_and_submit_transaction(
        account['id'],
        Amount.from_display_units(0.01, MOB),
        temp_account['main_address'],
    )
    tx_value = Amount.from_storage_units(
        transaction_log['value_map'][str(MOB.token_id)],
        MOB,
    )
    assert tx_value == Amount.from_display_units(0.01, MOB)
    assert transaction_log['output_txos'][0]['public_key'] == transaction_log['id']

    # Wait for the temporary account to sync.
    tx_index = int(transaction_log['submitted_block_index'])
    temp_status = await client.poll_account_status(temp_account['id'], tx_index + 1)

    # Check that the transaction has arrived.
    temp_balance = Amount.from_storage_units(
        temp_status['balance_per_token'][str(MOB.token_id)]['unspent'],
        MOB,
    )
    assert temp_balance == Amount.from_display_units(0.01, MOB)


async def test_send_transaction(client, source_account, account_factory):
    temp_account = await account_factory.create()
    await _test_send_transaction(client, source_account, temp_account)


@pytest.mark.skipif(
    (
        'MC_FOG_REPORT_URL' not in os.environ or
        'MC_FOG_AUTHORITY_SPKI' not in os.environ
    ),
    reason='Fog server not given.'
)
async def test_send_transaction_fog(client, source_account, account_factory):
    temp_fog_account = await account_factory.create_fog()
    await _test_send_transaction(client, source_account, temp_fog_account)


async def test_send_transaction_subaddress(client, source_account, account_factory):
    temp_account = await account_factory.create()

    # Create a new address for the temporary account.
    address = await client.assign_address_for_account(temp_account['id'])
    address = address['public_address_b58']

    # Send a transaction to the temporary account.
    transaction_log, _ = await client.build_and_submit_transaction(
        source_account['id'],
        Amount.from_display_units(0.01, MOB),
        address,
    )

    # Wait for the temporary account to sync.
    tx_index = int(transaction_log['submitted_block_index'])
    temp_status = await client.poll_account_status(temp_account['id'], tx_index + 1)
    print(temp_status)

    # Check that the transaction has arrived.
    temp_balance = Amount.from_storage_units(
        temp_status['balance_per_token'][str(MOB.token_id)]['unspent'],
        MOB,
    )
    assert temp_balance == Amount.from_display_units(0.01, MOB)

    # The subaddress balance also shows the transaction.
    subaddress_status = await client.get_address_status(address)
    subaddress_balance = Amount.from_storage_units(
        subaddress_status['balance_per_token'][str(MOB.token_id)]['unspent'],
        MOB,
    )
    assert subaddress_balance == Amount.from_display_units(0.01, MOB)


async def test_build_transaction_multiple_outputs(client, source_account, account_factory):
    temp_account_1 = await account_factory.create()
    temp_account_2 = await account_factory.create()

    tx_proposal, _ = await client.build_transaction(
        source_account['id'],
        {
            temp_account_1['main_address']: Amount.from_display_units(0.01, MOB),
            temp_account_2['main_address']: Amount.from_display_units(0.01, MOB),
        },
    )
    transaction_log = await client.submit_transaction(tx_proposal, source_account['id'])

    # Wait for the temporary accounts to sync.
    tx_index = int(transaction_log['submitted_block_index'])
    statuses = await asyncio.gather(
        client.poll_account_status(temp_account_1['id'], tx_index + 1),
        client.poll_account_status(temp_account_2['id'], tx_index + 1),
    )

    # Check that the transactions have arrived.
    for status in statuses:
        balance = Amount.from_storage_units(
            status['balance_per_token'][str(MOB.token_id)]['unspent'],
            MOB,
        )
        assert balance == Amount.from_display_units(0.01, MOB)


async def test_build_burn_transaction(client, source_account, fees):
    # Check the initial balance.
    status = await client.get_account_status(source_account['id'])
    initial_balance = Amount.from_storage_units(
        status['balance_per_token'][str(MOB.token_id)]['unspent'],
        MOB,
    )

    # Build and submit a burn transaction.
    burn_amount = Amount.from_display_units(0.0004, MOB)
    tx_proposal, _ = await client.build_burn_transaction(
        source_account['id'],
        burn_amount
    )
    transaction_log = await client.submit_transaction(tx_proposal, source_account['id'])

    # Wait for the account to sync.
    tx_index = int(transaction_log['submitted_block_index'])
    status = await client.poll_account_status(source_account['id'], tx_index + 1)

    # Check that the funds are missing.
    final_balance = Amount.from_storage_units(
        status['balance_per_token'][str(MOB.token_id)]['unspent'],
        MOB,
    )
    assert final_balance == initial_balance - burn_amount - fees[MOB]
