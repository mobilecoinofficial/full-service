import asyncio
from fullservice import FullServiceAPIv2 as v2 
import requests

async def smoke_test():
    fs = v2()
    data = await fs.version()
    slack = '{"text": "%s"}' % data
    requests.post(url='https://hooks.slack.com/services/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', headers={"Content-type": "application/json"}, data=slack)
    
if __name__ == "__main__":
    asyncio.run(smoke_test())
