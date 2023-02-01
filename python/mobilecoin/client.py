import asyncio
import logging
import aiohttp
import json

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

    async def create_account(self, name=None):
        r = await self._req({
            "method": "create_account",
            "params": {
                "name": name,
            }
        })
        return r['account']

    async def import_account(
        self,
        mnemonic,
        key_derivation_version=2,
        name=None,
        first_block_index=None,
        next_subaddress_index=None,
        fog_keys=None,
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
        if fog_keys is not None:
            params.update(fog_keys)

        r = await self._req({
            "method": "import_account",
            "params": params
        })
        return r['account']

    async def remove_account(self, account_id):
        return await self._req({
            "method": "remove_account",
            "params": {"account_id": account_id}
        })

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
