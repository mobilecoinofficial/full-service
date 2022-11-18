import asyncio
from fullservice import FullServiceAPIv2 as v2 
import requests

async def smoke_test():
    fs = v2()
    #basic proof of concept, returns version
    data = await fs.version()
    
    slack = '{"text": "%s"}' % data
    #slack URL to post response to not parsed or made clean yet
    requests.post(url='', headers={"Content-type": "application/json"}, data=slack)
    
if __name__ == "__main__":
    asyncio.run(smoke_test())
