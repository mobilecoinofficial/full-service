import argparse
import asyncio
import json
import subprocess
import sys

demo = True

repo_root_dir = subprocess.check_output("git rev-parse --show-toplevel", shell=True).decode("utf8").strip()
sys.path.append("{}/python-library".format(repo_root_dir))

from fullservice import FullServiceAPIv2 as v2
from FSDataObjects import Response, Account 

sleepy_time = 3
default_config_path = "./test_config.json"
config = []
mnemonics=[]
account_ids = []

fs = v2()

PICO_MOB= 1 000 000 000 000

def demo(*args)
    if demo:
        input(print(args))



demo("Welcome to Alice's journey with full-service \n --------------------------------------------")
demo("Alice is already using the Moby app. As the most tech savvy person in her family, she's decided to use full-service to help manage her loved ones' wallets")
demo("Earlier today, Alice followed the installation instructions to setup and run full-serivce")
demo("She already has a wallet with some funds, and will use her mnemonic to setup the wallet on full-service too")



