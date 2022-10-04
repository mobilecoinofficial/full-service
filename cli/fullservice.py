# Copyright (c) 2022 MobileCoin, Inc.


import asyncio
from queue import Full
from unittest import result
from urllib import request
import aiohttp

import http.client
import json
import os
import pathlib
import shutil
import subprocess
import time

import logging
import ssl
import base64

from typing import Any, Optional

# from . import constants
import forest_utils as utils
from typing import Tuple
from urllib.parse import urlparse

if not utils.get_secret("ROOTCRT"):
    ssl_context: Optional[ssl.SSLContext] = None
else:
    ssl_context = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
    root = open("rootcrt.pem", "wb")
    root.write(base64.b64decode(utils.get_secret("ROOTCRT")))
    root.flush()
    client = open("client.full.pem", "wb")
    client.write(base64.b64decode(utils.get_secret("CLIENTCRT")))
    client.flush()

    ssl_context.load_verify_locations("rootcrt.pem")
    ssl_context.verify_mode = ssl.CERT_REQUIRED
    ssl_context.load_cert_chain(certfile="client.full.pem")


class FullServiceAPIv1():
    def __init__(self, remove_wallet_and_ledger=False):
        super().__init__()
        self.full_service_process = None
        self.account_map = None
        self.account_ids = None
        self.request_count = 0
        self.remove_wallet_and_ledger = remove_wallet_and_ledger
        url = ()
        if not url:
            url = (
                utils.get_secret("FULL_SERVICE_URL") or "http://localhost:9090/"
            ).removesuffix("/wallet") + "wallet"
        logging.info("full-service url: %s", url)
        self.url = url
        self.wallet_path = pathlib.Path("/tmp/wallet-db")
        self.ledger_path = pathlib.Path("/tmp/ledger-db")

    async def req(self, method: str, **params: Any) -> dict:
        logging.info("request: %s", method)
        response_data = await self.request({"method": method, "params": params})
        if "error" in result:
            logging.error(result)
        return result

    # return the result field of the request
    ### is this a breaking change with unittests?
    async def request(self, request_data):
        self.request_count += 1
        request_data = {"jsonrpc": "2.0", "id": self.request_count, **request_data}
        print(f"request data: {request_data}")
        async with aiohttp.TCPConnector(ssl=ssl_context) as conn:
            async with aiohttp.ClientSession(connector=conn) as sess:
                # this can hang (forever?) if there's no full-service at that url
                async with sess.post(
                    self.url,
                    data=json.dumps(request_data),
                    headers={"Content-Type": "application/json"},
                ) as resp:
                    print(resp.json)
                    return await resp.json()

    # check if full service is synced within margin
    def sync_status(self, eps=5) -> bool:
        # ping network
        try:
            r = self.req({"method": "get_network_status"})
        except ConnectionError as e:
            print(e)
            return False

        # network offline
        if int(r["network_status"]["network_block_height"]) == 0:
            return False

        # network online
        network_block_height = int(r["network_status"]["network_block_height"])
        local_block_height = int(r["network_status"]["local_block_height"])

        # network is acceptably synced
        return network_block_height - local_block_height < eps

    def sync_full_service_to_network(self, mc_network):
        network_synced = False
        count = 0
        attempt_limit = 100
        while network_synced is False and count < attempt_limit:
            count += 1
            network_synced = self.sync_status()
            if count % 10 == 0:
                print(f"attempt: {count}/{attempt_limit}")
            time.sleep(1)
        if count >= attempt_limit:
            raise Exception(f"Full service sync failed after {attempt_limit} attempts")
        print("Full service synced")


    # retrieve wallet status
    async def get_wallet_status(self):
        r = await self.req({"method": "get_wallet_status"})
        return r["wallet_status"]

    class Account:
        def recover(self, mnemonic) -> bool:
            print(f"importing full service account {mnemonic}")
            params = {
                "mnemonic": mnemonic,
                "key_derivation_version": "2",
            }
            r = FullServiceAPIv1.req({"method": "import_account", "params": params})

            if "error" in r:
                # If we failed due to a unique constraint, it means the account already exists
                return (
                    "Diesel Error: UNIQUE constraint failed"
                    in r["error"]["data"]["details"]
                )
            return True

        # retrieve all accounts full service is aware of
        def get_all(self) -> Tuple[list, dict]:
            r = FullServiceAPIv1.req({"method": "get_all_accounts"})
            print(r)
            return (r["account_ids"], r["account_map"])

        # retrieve information about account
        def get_status(self, account_id: str):
            params = {"account_id": account_id}
            r = FullServiceAPIv1.req({"method": "get_account_status", "params": params})
            return r

    # build and submit a transaction from `account_id` to `to_address` for `amount` of pmob
    def send_transaction(self, account_id, to_address, amount):
        params = {
            "account_id": account_id,
            "addresses_and_values": [(to_address, amount)],
        }
        r = self.req(
            {
                "method": "build_and_submit_transaction",
                "params": params,
            }
        )
        print(r)
        return r["transaction_log"]


class FullServiceAPIv2:
    def stub():
        pass

    

