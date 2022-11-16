# verify that the transaction went through
#   the mob went through
#   the transaction log updatedx
# Ideally all of the endpoints (v2) that actually hit the mobilecoin network
# 
#     get_network_status
#     get_wallet_status
#     build, build_and_submit, build_split_txo .. etc

import sys
import os
sys.path.append(os.path.abspath("../cli"))

from fullservice import FullServiceAPIv2 as v2 
import asyncio
import json
import subprocess
from pathlib import Path

with open('config') as json_file:
    config = json.load(json_file)


async def main():
    fs = v2()

    network_status = await fs.get_network_status()
    print(network_status)
    

if __name__ == '__main__':
    asyncio.run(main())