import aiohttp
import asyncio
from time import perf_counter


async def main():
    c = StressClient()
    await test_account_create(c)
    await test_subaddresses(c)


async def test_account_create(c, n=10):
    accounts = await c.get_all_accounts()
    num_accounts_before = len(accounts)

    account_ids = []
    async def create_one(i):
        account = await c.create_account(str(i))
        account_ids.append(account['account_id'])

    with Timer() as timer:
        await asyncio.gather(*[
            create_one(i)
            for i in range(1, n+1)
        ])

    accounts = await c.get_all_accounts()
    assert len(accounts) == num_accounts_before + n

    await asyncio.gather(*[
        c.remove_account(account_id)
        for account_id in account_ids
    ])

    print('Created {} accounts in {:.3f}s'.format(n, timer.elapsed))


async def test_subaddresses(c, n=10):
    account = await c.create_account()
    account_id = account['account_id']

    addresses = await c.get_addresses_for_account(account_id)
    assert len(addresses) == 2

    with Timer() as timer:
        await asyncio.gather(*[
            c.assign_address(account_id, str(i))
            for i in range(1, n+1)
        ])

    addresses = await c.get_addresses_for_account(account_id)
    assert len(addresses) == 2 + n

    await c.remove_account(account_id)

    print('Created {} addresses in {:.3f}s'.format(n, timer.elapsed))


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
        try:
            return r['result']
        except KeyError:
            print(r)
            raise

    async def get_all_accounts(self):
        r = await self._req({"method": "get_all_accounts"})
        return r['account_map']

    async def create_account(self, name=''):
        r = await self._req({
            "method": "create_account",
            "params": {"name": name}
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


class Timer:
    def __enter__(self):
        self._start_time = perf_counter()
        return self

    def __exit__(self, *_):
        end_time = perf_counter()
        self.elapsed = end_time - self._start_time


if __name__ == '__main__':
    asyncio.run(main())
