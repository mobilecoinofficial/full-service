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
async def account_id(client):
    """
    Import the account specified by the MC_WALLET_FILE environment variable,
    and return its account id.
    """
    wallet_file = os.environ['MC_WALLET_FILE']
    with open(wallet_file) as f:
        data = json.load(f)

    account_id = None
    try:
        # Import an account and wait for it to sync. 
        account = await client.import_account(
            mnemonic=data['mnemonic'],
            first_block_index=data['first_block_index'],
        )
        account_id = account['id']
        await client.poll_account_status(account_id, timeout=60)

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
    yield account_id


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


async def test_wallet_status(client):
    wallet_status = await client.get_wallet_status()
    assert sorted(wallet_status.keys()) == [
        'balance_per_token',
        'is_synced_all',
        'local_block_height',
        'min_synced_block_index',
        'network_block_height',
    ]


async def test_account_status(client, account_id):
    status = await client.get_account_status(account_id)
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
