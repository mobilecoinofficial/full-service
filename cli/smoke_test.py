import asyncio
from fullservice import FullServiceAPIv2 as v2 
import requests
import forest_utils

async def smoke_test():
    fs = v2()
    await fs.import_account(mnemonic=f"{forest_utils.get_secret('mnemonic')}")
    acc_id = None  # fetch the account id here
    data = await fs.build_and_submit_transaction(account_id=acc_id, recipient_public_address=f"{forest_utils.get_secret('recipient_public_address')}", amount='{"value": "0001", "token_id": "0"}')
    slack = '{"text": "%s"}' % data
    #slack URL to post response to not parsed or made clean yet
    requests.post(url='', headers={"Content-type": "application/json"}, data=slack)
    
if __name__ == "__main__":
    asyncio.run(smoke_test())
