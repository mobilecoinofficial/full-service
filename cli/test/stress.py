import aiohttp
import asyncio
import time


async def main():
        c = StressClient()

        account = await c.create_account()
        account_id = account['account_id']

        await test_addresses(c, account_id, n=100)

        await c.remove_account(account_id)


async def test_addresses(c, account_id, n=10):
    addresses = await c.get_addresses_for_account(account_id)
    assert len(addresses) == 2

    start = time.time()
    await asyncio.gather(*[
        c.assign_address(account_id, str(i))
        for i in range(1, n+1)
    ])
    end = time.time()

    addresses = await c.get_addresses_for_account(account_id)
    assert len(addresses) == n + 2

    print('Created {} addresses in {:.3f}s'.format(n, end - start))


class StressClient:

    async def _req(self, request_data):
        default_params = {
            "jsonrpc": "2.0",
            "api_version": "2",
            "id": 1,
        }
        request_data = {**request_data, **default_params}
        async with aiohttp.ClientSession() as session:
            async with session.post('http://localhost:9090/wallet', json=request_data) as response:
                r = await response.json()
        return r['result']

    async def create_account(self):
        r = await self._req({
            "method": "create_account",
            "params": {"name": ""}
        })
        return r['account']

    async def remove_account(self, account_id):
        return await self._req({
            "method": "remove_account",
            "params": {"account_id": account_id}
        })

    async def assign_address(self, account_id, name=''):
        return await self._req({
            "method": "assign_address_for_account",
            "params": {
                "account_id": account_id,
                "metadata": name,
            },
        })

    async def get_addresses_for_account(self, account_id):
        r = await self._req({
            "method": "get_addresses_for_account",
            "params": {
                "account_id": account_id,
                "offset": "0",
                "limit": "1000",
            },
        })
        return r['address_map']


loop = asyncio.get_event_loop()
loop.run_until_complete(main())
