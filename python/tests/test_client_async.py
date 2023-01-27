import asyncio
from mobilecoin.client_async import ClientAsync, ClientSync
from pprint import pprint


def main():
    asyncio.run(test_async())
    test_sync()


async def test_async():
    async with ClientAsync() as c:
        await test_account(c)


async def test_account(c):
    pprint(await c.get_accounts())


def test_sync():
    c = ClientSync()
    test_account_sync(c)


def test_account_sync(c):
    pprint(c.get_accounts())


if __name__ == '__main__':
    main()

