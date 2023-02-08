import asyncio
import os
import json
import re
import pytest

from mobilecoin.client import (
    ClientAsync,
    WalletAPIError,
    log as client_log,
)
from mobilecoin.token import get_token, Amount

MOB = get_token('MOB')
EUSD = get_token('EUSD')


# Enable debug logging during unittests.
client_log.setLevel('DEBUG')


# In order to import just one wallet for the whole test session, we need to set the
# asyncio-pytest event loop to session scope.
@pytest.fixture(scope="session")
def event_loop():
    policy = asyncio.get_event_loop_policy()
    loop = policy.new_event_loop()
    yield loop
    loop.close()


@pytest.fixture(scope='session')
async def client():
    async with ClientAsync() as client:
        yield client


@pytest.fixture(scope='session')
async def fees(client):
    # Get the default fee amounts.
    fees = {}
    network_status = await client.get_network_status()
    for token_id, value in network_status['fees'].items():
        try:
            amount = Amount.from_storage_units(value, token_id)
            fees[amount.token] = amount
        except KeyError:
            pass  # Ignore unknown tokens.
    return fees


@pytest.fixture(scope='session')
async def account(client):
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
        account = await client.import_account(
            mnemonic=data['mnemonic'],
            first_block_index=data['first_block_index'],
        )
        account_id = account['id']

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
    status = await client.poll_account_status(account_id, timeout=60)
    initial_balance = Amount.from_storage_units(
        status['balance_per_token'][str(MOB.token_id)]['unspent'],
        MOB,
    )
    assert initial_balance >= Amount.from_display_units(0.1, MOB)

    return status['account']


@pytest.fixture
async def temp_account(client, account, fees):
    temp_account = await client.create_account()

    yield temp_account

    await _clean_up_temp_account(client, account, temp_account, fees)


@pytest.fixture
async def temp_fog_account(client, account, fees):
    temp_account = await client.create_account(
        fog_report_url=os.environ['MC_FOG_REPORT_URL'],
        fog_authority_spki=os.environ['MC_FOG_AUTHORITY_SPKI'],
    )

    yield temp_account

    await _clean_up_temp_account(client, account, temp_account, fees)



async def _clean_up_temp_account(client, account, temp_account, fees):
    # Send all funds back to the main account.
    tx_index = None
    status = await client.get_account_status(temp_account['id'])
    for token_id, value in status['balance_per_token'].items():
        amount = Amount.from_storage_units(value['unspent'], token_id)
        if amount >= fees[amount.token]:
            send_amount = amount - fees[amount.token]
            transaction_log, _ = await client.build_and_submit_transaction(
                temp_account['id'],
                send_amount,
                account['main_address'],
            )
            tx_index = int(transaction_log['submitted_block_index'])

    # Ensure the temporary account has no funds remaining.
    if tx_index is not None:
        status = await client.poll_account_status(temp_account['id'], tx_index + 1)
        for balance in status['balance_per_token'].values():
            assert int(balance['unspent']) == 0

    # Delete the temporary account.
    await client.remove_account(temp_account['id'])


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
        'network_block_height',
    ]


async def test_network_status_fees(fees):
    assert sorted( t.short_code for t in fees.keys() ) == ['MOB', 'eUSD']
    assert all( isinstance(a, Amount) for a in fees.values() )


async def test_get_block(client):
    network_status = await client.get_network_status()
    last_block_index = int(network_status['network_block_height']) - 1
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


async def test_account_status(client, account):
    status = await client.get_account_status(account['id'])
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
        'name',
        'next_block_index',
        'next_subaddress_index',
        'recovery_mode',
        'view_only',
    ]


async def test_send_transaction_self(client, account, fees):
    # Check the initial balance.
    status = await client.get_account_status(account['id'])
    initial_balance = Amount.from_storage_units(
        status['balance_per_token'][str(MOB.token_id)]['unspent'],
        MOB,
    )

    # Send a transaction from the account back to itself.
    transaction_log, _ = await client.build_and_submit_transaction(
        account['id'],
        Amount.from_display_units(0.01, MOB),
        account['main_address'],
    )
    tx_value = Amount.from_storage_units(
        transaction_log['value_map'][str(MOB.token_id)],
        MOB,
    )
    assert tx_value == Amount.from_display_units(0.01, MOB)

    # Wait for the account to sync.
    tx_index = int(transaction_log['submitted_block_index'])
    status = await client.poll_account_status(account['id'], tx_index + 1)

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

    # Wait for the temporary account to sync.
    tx_index = int(transaction_log['submitted_block_index'])
    temp_status = await client.poll_account_status(temp_account['id'], tx_index + 1)

    # Check that the transaction has arrived.
    temp_balance = Amount.from_storage_units(
        temp_status['balance_per_token'][str(MOB.token_id)]['unspent'],
        MOB,
    )
    assert temp_balance == Amount.from_display_units(0.01, MOB)


async def test_send_transaction(client, account, temp_account):
    await _test_send_transaction(client, account, temp_account)


@pytest.mark.skipif(
    (
        'MC_FOG_REPORT_URL' not in os.environ or
        'MC_FOG_AUTHORITY_SPKI' not in os.environ
    ),
    reason='Fog server not given.'
)
async def test_send_transaction_fog(client, account, temp_fog_account):
    await _test_send_transaction(client, account, temp_fog_account)


async def test_send_transaction_subaddress(client, account, temp_account):
    # Create a new address for the temporary account.
    address = await client.assign_address_for_account(temp_account['id'])
    address = address['public_address_b58']

    # Send a transaction to the temporary account.
    transaction_log, _ = await client.build_and_submit_transaction(
        account['id'],
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
