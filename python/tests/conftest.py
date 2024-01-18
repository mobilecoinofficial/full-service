import asyncio
import pytest
import os
import json
import re
from mnemonic import Mnemonic

from mobilecoin.client import (
    ClientAsync,
    WalletAPIError,
    log as client_log,
)
from mobilecoin.token import get_token, Amount

MOB = get_token('MOB')

# Enable debug logging during unittests.
client_log.setLevel('DEBUG')

# Load source wallet file.
with open(os.environ['MC_WALLET_FILE']) as f:
    WALLET_DATA = json.load(f)


@pytest.fixture(scope="session")
def event_loop():
    # In order to import just one wallet for the whole test session, we need to set the
    # asyncio-pytest event loop to session scope.
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
    """Get the default fee amounts."""
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
async def source_account(client):
    """
    Import the account specified by the MC_WALLET_FILE environment variable,
    and return its account id.
    """
    account_id = None
    try:
        # Import an account.
        account = await client.import_account(
            mnemonic=WALLET_DATA['mnemonic'],
            first_block_index=WALLET_DATA['first_block_index'],
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


@pytest.fixture(scope='session')
async def account_factory(client, source_account, fees):
    network_status = await client.get_network_status()
    local_block_height = int(network_status['local_block_height'])

    class AccountFactory:
        def __init__(self):
            self.temp_accounts = []

        def next_mnemonic(self):
            # Increment the source account mnemonic by one for each temp account we create.
            return increment_mnemonic(WALLET_DATA['mnemonic'], len(self.temp_accounts) + 1)

        async def create(self):
            # Create the temporary account.
            temp_account = await client.import_account(
                mnemonic=self.next_mnemonic(),
                first_block_index=local_block_height - 1000,
            )
            self.temp_accounts.append(temp_account)
            return temp_account

        async def create_fog(self):
            fog_info = {
                "report_url": os.environ["MC_FOG_REPORT_URL"],
                "authority_spki": os.environ["MC_FOG_AUTHORITY_SPKI"],
            }
            temp_fog_account = await client.import_account(
                mnemonic=self.next_mnemonic(),
                first_block_index=local_block_height - 1000,
                fog_info=fog_info,
            )
            self.temp_accounts.append(temp_fog_account)
            return temp_fog_account

        async def cleanup(self):
            await asyncio.gather(*(
                clean_up_temp_account(client, source_account, temp_account, fees)
                for temp_account in self.temp_accounts
            ))

    account_factory = AccountFactory()
    yield account_factory
    await account_factory.cleanup()


def increment_mnemonic(mnemonic, increment):
    m = Mnemonic('english')
    entropy = m.to_entropy(mnemonic)
    number = int.from_bytes(entropy, 'big')
    number += increment
    next_entropy = number.to_bytes(len(entropy), 'big')
    return m.to_mnemonic(next_entropy)


async def clean_up_temp_account(client, source_account, temp_account, fees):
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
                source_account['main_address'],
            )
            tx_index = int(transaction_log['submitted_block_index'])

    # Ensure the temporary account has no funds remaining.
    if tx_index is not None:
        status = await client.poll_account_status(temp_account['id'], tx_index + 1)
        for balance in status['balance_per_token'].values():
            assert int(balance['unspent']) == 0

    # Delete the temporary account.
    await client.remove_account(temp_account['id'])
