import asyncio 
from fullservice import FullServiceAPIv2 as v2

async def main():
    fs = v2()
    accounts_before = await fs.get_accounts()
    count = accounts_before['result']['account_ids']
    await fs.create_account()
    accounts_after = await fs.get_accounts()
    if len(accounts_after['result']['account_ids']) == len(count) + 1:
        print("Account created successfully!")
    else:
        print("Account creation failed")
asyncio.run(main())