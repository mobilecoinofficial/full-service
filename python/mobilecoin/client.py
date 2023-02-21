import asyncio
import logging
import aiohttp
import json
import time
from typing import Optional

from mobilecoin.token import get_token, Amount


log = logging.getLogger('client_async')

DEFAULT_URL = 'http://127.0.0.1:9090/wallet/v2'
MAX_TOMBSTONE_BLOCKS = 100


class WalletAPIError(Exception):
    def __init__(self, response):
        self.response = response


class ClientAsync:
    def __init__(self, url=None):
        if url is None:
            url = DEFAULT_URL
        self.url = url
        self._query_count = 0

        self.session = aiohttp.ClientSession()

    def __enter__(self):
        raise TypeError("Use async with instead")

    def __exit__(self, *args):
        del args

    async def __aenter__(self):
        return self

    async def __aexit__(self, *args):
        del args
        await self.session.close()

    async def _req(self, request_data):
        default_params = {
            "jsonrpc": "2.0",
            "id": 1,
        }
        request_data = {**request_data, **default_params}
        log.debug(f'POST {json.dumps(request_data, indent=4)}')
        async with self.session.post(self.url, json=request_data) as response:
            r_json = await response.text()
        r = json.loads(r_json)
        log.debug(f'Response: {json.dumps(r, indent=4)}')
        try:
            return r['result']
        except KeyError:
            raise WalletAPIError(r)

    async def version(self):
        return await self._req({"method": "version"})

    async def get_network_status(self):
        r = await self._req({"method": "get_network_status"})
        return r['network_status']

    async def get_block(self, block_index):
        r = await self._req({
            "method": "get_block",
            "params": {
                "block_index": str(block_index),
            }
        })
        return r['block'], r['block_contents']

    async def get_wallet_status(self):
        r = await self._req({"method": "get_wallet_status"})
        return r['wallet_status']

    async def get_accounts(self, offset=None, limit=None):
        r = await self._req({
            "method": "get_accounts",
            "params": {
                "offset": offset,
                "limit": limit,
            }
        })
        return r['account_map']

    async def get_account_status(self, account_id):
        return await self._req({
            "method": "get_account_status",
            "params": {
                "account_id": account_id,
            },
        })

    async def create_account(
        self,
        name=None,
        fog_report_url=None,
        fog_authority_spki=None,
    ):
        params = {"name": name}
        if fog_report_url is not None and fog_authority_spki is not None:
            params['fog_info'] = {
                "report_url": fog_report_url,
                "report_id": "",
                "authority_spki": fog_authority_spki,
            }
        r = await self._req({"method": "create_account", "params": params})
        return r['account']

    async def import_account(
        self,
        mnemonic,
        key_derivation_version=2,
        name=None,
        first_block_index=None,
        next_subaddress_index=None,
        fog_report_url=None,
        fog_authority_spki=None,
    ):
        params = {
            'mnemonic': mnemonic,
            'key_derivation_version': str(int(key_derivation_version)),
        }
        if name is not None:
            params['name'] = name
        if first_block_index is not None:
            params['first_block_index'] = str(int(first_block_index))
        if next_subaddress_index is not None:
            params['next_subaddress_index'] = str(int(next_subaddress_index))
        if fog_report_url is not None and fog_authority_spki is not None:
            params['fog_info'] = {
                "report_url": fog_report_url,
                "report_id": "",
                "authority_spki": fog_authority_spki,
            }

        r = await self._req({
            "method": "import_account",
            "params": params
        })
        return r['account']

    async def export_account_secrets(self, account_id):
        r = await self._req({
            "method": "export_account_secrets",
            "params": {"account_id": account_id}
        })
        return r['account_secrets']

    async def update_account_name(self, account_id, name):
        r = await self._req({
            "method": "update_account_name",
            "params": {
                "account_id": account_id,
                "name": name,
            }
        })
        return r['account']

    async def remove_account(self, account_id):
        return await self._req({
            "method": "remove_account",
            "params": {"account_id": account_id}
        })

    async def get_addresses(self, account_id, offset=0, limit=1000):
        r = await self._req({
            "method": "get_addresses",
            "params": {
                "account_id": account_id,
                "offset": int(offset),
                "limit": int(limit),
            },
        })
        return r['address_map']

    async def get_address_status(self, address):
        return await self._req({
            "method": "get_address_status",
            "params": {
                "address": address,
            },
        })

    async def assign_address_for_account(self, account_id, metadata=None):
        if metadata is None:
            metadata = ''

        r = await self._req({
            "method": "assign_address_for_account",
            "params": {
                "account_id": account_id,
                "metadata": metadata,
            },
        })
        return r['address']

    async def get_transaction_logs(
        self,
        account_id,
        min_block_index=None,
        max_block_index=None,
        offset=None,
        limit=None,
    ):
        r = await self._req({
            "method": "get_transaction_logs",
            "params": {
                "account_id": account_id,
                "min_block_index": min_block_index,
                "max_block_index": max_block_index,
                "offset": offset,
                "limit": limit,
            },
        })
        return r['transaction_log_map']

    @staticmethod
    def _build_transaction_params(
        account_id,
        addresses_and_amounts,
        tombstone_block=None,
        fee=None,
    ):
        params = {
            "account_id": account_id,
            "addresses_and_amounts": [],
        }
        for (address, amount) in addresses_and_amounts.items():
            amount_json = {
                "value": str(amount.value),
                "token_id": str(amount.token.token_id),
            }
            params['addresses_and_amounts'].append((address, amount_json))
        if fee is not None:
            params['fee_value'] = str(fee.value)
            params['fee_token_id'] = str(fee.token.token_id)
        if tombstone_block is not None:
            params['tombstone_block'] = str(int(tombstone_block))
        return params

    async def build_transaction(self, *args, **kwargs):
        r = await self._req({
            "method": "build_transaction",
            "params": self._build_transaction_params(*args, **kwargs),
        })
        return r['tx_proposal'], r['transaction_log_id']

    async def build_unsigned_transaction(self, *args, **kwargs):
        r = await self._req({
            "method": "build_unsigned_transaction",
            "params": self._build_transaction_params(*args, **kwargs),
        })
        return r

    async def build_burn_transaction(self, account_id, amount, redemption_memo_hex=None):
        r = await self._req({
            "method": "build_burn_transaction",
            "params": {
                "account_id": account_id,
                "amount": {
                    "value": str(amount.value),
                    "token_id": str(amount.token.token_id)
                },
                "redemption_memo_hex": redemption_memo_hex,
            }
        })
        return r['tx_proposal'], r['transaction_log_id']

    async def submit_transaction(self, tx_proposal, account_id=None):
        r = await self._req({
            "method": "submit_transaction",
            "params": {
                "tx_proposal": tx_proposal,
                "account_id": account_id,
            },
        })
        return r['transaction_log']

    async def build_and_submit_transaction(
        self,
        account_id,
        amount,
        to_address,
        fee=None,
    ):
        params = {
            "account_id": account_id,
            "recipient_public_address": to_address,
            "amount": {
                "value": str(amount.value),
                "token_id": str(amount.token.token_id)
            }
        }
        if fee is not None:
            params['fee_value'] = str(fee.value)
            params['fee_token_id'] = str(fee.token.token_id)

        r = await self._req({
            "method": "build_and_submit_transaction",
            "params": params,
        })
        return r['transaction_log'], r['tx_proposal']

    async def import_view_only_account(self, params):
        r = await self._req({
            "method": "import_view_only_account",
            "params": params,
        })
        return r['account']

    async def create_view_only_account_sync_request(self, account_id):
        r = await self._req({
            "method": "create_view_only_account_sync_request",
            "params": {
                "account_id": account_id,
            },
        })
        return r

    async def sync_view_only_account(self, params):
        r = await self._req({
            "method": "sync_view_only_account",
            "params": params,
        })
        return r

    # Polling utility functions.

    @staticmethod
    async def poll(func, delay=1.0, timeout=10):
        """
        Repeatedly call the given function until it returns a result.
        """
        start = time.monotonic()
        while True:
            elapsed = time.monotonic() - start
            if elapsed >= timeout:
                raise TimeoutError('Polling timed out before succeeding.')
            try:
                result = await func()
            except ConnectionError:
                result = None
            if result is not None:
                return result
            await asyncio.sleep(delay)

    async def poll_account_status(self, account_id, min_block_height=None, **kwargs):
        """
        Poll the account status endpoint for this account, until it reports as synced.

        If the `min_block_height` argument is given, don't return until it is
        synced above the given block height. This is a way to wait for a
        submitted transaction to complete.
        """
        async def func():
            status = await self.get_account_status(account_id)
            account_block = int(status['account']['next_block_index']) 
            network_block = int(status['local_block_height'])
            synced = (account_block >= network_block)
            if synced:
                if (
                    min_block_height is None
                    or account_block >= min_block_height
                ):
                    return status
        return await self.poll(func, **kwargs)


class ClientSync:
    """
    Convenience class to make it easier to use the client from synchronous
    code. Any time we call a method of this client, it constructs an inner
    asynchronous client, and calls the underlying async method.
    """
    def __init__(self, url=None):
        self.url = url

    def __getattr__(self, name):
        def inner(*args, **kwargs):
            result = None
            async def runner():
                async with ClientAsync() as c:
                    nonlocal result
                    method = getattr(c, name)
                    result = await method(*args, **kwargs)
            asyncio.run(runner())
            return result
        return inner
