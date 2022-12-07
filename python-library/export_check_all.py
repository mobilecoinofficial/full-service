from fullservice import FullServiceAPIv2 as v2
import asyncio
from rich import print 

fs = v2()

async def get():
    accounts = await fs.get_accounts()
    result = accounts.get("result").get("account_ids")
    for account_id in result:
        res_secrets = await fs.export_account_secrets(account_id)
        res_status = await fs.get_account_status(account_id)
        print(
            "\n",
            res_secrets.get("result").get("account_secrets").get("mnemonic"), "\nID:", account_id,
            res_status.get("result").get("balance_per_token"),
        )
        
async def clean():
    print("Cleaning up accounts")
    accounts = await fs.get_accounts()
    result = accounts.get("result").get("account_ids")
    for account_id in result:
        await fs.remove_account(account_id)
        return print("Done cleaning up accounts")

if "__main__" == __name__:
    asyncio.run(get())