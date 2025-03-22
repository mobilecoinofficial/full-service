from decimal import Decimal
import http.client
import json
import os
import time
from urllib.parse import urlparse

from mobilecoin.token import get_token

MOB = get_token('MOB')

DEFAULT_HOST = 'http://127.0.0.1'
DEFAULT_PORT = 9090

MAX_TOMBSTONE_BLOCKS = 20160


class WalletAPIError(Exception):
    def __init__(self, response):
        self.response = response


class Client:
    """
    *DEPRECATED*
    This is a legacy implementation of the full-service API, version 1.
    This version of the API does not support multiple token types. Please do
    not use this for new development.
    """

    REQ_PATH = '/wallet'

    def __init__(self, host=None, port=None, verbose=False):
        if host is None:
            host = os.environ.get('MC_FULL_SERVICE_HOST', DEFAULT_HOST)
        if port is None:
            port = os.environ.get('MC_FULL_SERVICE_PORT', DEFAULT_PORT)
        self.url = f'{host}:{port}' + self.REQ_PATH

        self.verbose = verbose
        self._query_count = 0

    def _req(self, request_data):
        default_params = {
            "jsonrpc": "2.0",
            "api_version": "2",
            "id": 1,
        }
        request_data = {**request_data, **default_params}

        if self.verbose:
            print('POST', self.url)
            print(json.dumps(request_data, indent=2))
            print()

        try:
            parsed_url = urlparse(self.url)
            connection = http.client.HTTPConnection(parsed_url.netloc)
            connection.request('POST', parsed_url.path, json.dumps(request_data), {'Content-Type': 'application/json'})
            r = connection.getresponse()

        except ConnectionError:
            raise ConnectionError(f'Could not connect to wallet server at {self.url}.')

        raw_response = None
        try:
            raw_response = r.read()
            response_data = json.loads(raw_response)
        except ValueError:
            raise ValueError('API returned invalid JSON:', raw_response)

        if self.verbose:
            print(r.status, http.client.responses[r.status])
            print(len(raw_response), 'bytes')
            print(json.dumps(response_data, indent=2))
            print()

        # Check for errors and unwrap result.
        try:
            result = response_data['result']
        except KeyError:
            raise WalletAPIError(response_data)

        self._query_count += 1

        return result

    def create_account(
        self,
        name=None,
        fog_report_url=None,
        fog_authority_spki=None,
    ):
        params = {"name": name}
        if fog_report_url is not None and fog_authority_spki is not None:
            params['fog_info'] = {
                "report_url": fog_report_url,
                "report_id": "",
                "authority_spki": fog_authority_spki,
            }
        r = self._req({"method": "create_account", "params": params})
        return r['account']

    def import_account(self, mnemonic, key_derivation_version=2, name=None, first_block_index=None, next_subaddress_index=None, fog_keys=None):
        params = {
            'mnemonic': mnemonic,
            'key_derivation_version': str(int(key_derivation_version)),
        }
        if name is not None:
            params['name'] = name
        if first_block_index is not None:
            params['first_block_index'] = str(int(first_block_index))
        if next_subaddress_index is not None:
            params['next_subaddress_index'] = str(int(next_subaddress_index))
        if fog_keys is not None:
            params.update(fog_keys)

        r = self._req({
            "method": "import_account",
            "params": params
        })
        return r['account']

    def import_account_from_legacy_root_entropy(self, legacy_root_entropy, name=None, first_block_index=None, next_subaddress_index=None, fog_keys=None):
        params = {
            'entropy': legacy_root_entropy,
        }
        if name is not None:
            params['name'] = name
        if first_block_index is not None:
            params['first_block_index'] = str(int(first_block_index))
        if next_subaddress_index is not None:
            params['next_subaddress_index'] = str(int(next_subaddress_index))
        if fog_keys is not None:
            params.update(fog_keys)

        r = self._req({
            "method": "import_account_from_legacy_root_entropy",
            "params": params
        })
        return r['account']

    def import_view_only_account(self, params):
        r = self._req({
            "method": "import_view_only_account",
            "params": params,
        })
        return r['account']

    def get_account(self, account_id):
        r = self._req({
            "method": "get_account",
            "params": {"account_id": account_id}
        })
        return r['account']

    def get_all_accounts(self):
        r = self._req({"method": "get_all_accounts"})
        return r['account_map']

    def update_account_name(self, account_id, name):
        r = self._req({
            "method": "update_account_name",
            "params": {
                "account_id": account_id,
                "name": name,
            }
        })
        return r['account']

    def remove_account(self, account_id):
        return self._req({
            "method": "remove_account",
            "params": {"account_id": account_id}
        })

    def export_account_secrets(self, account_id):
        r = self._req({
            "method": "export_account_secrets",
            "params": {"account_id": account_id}
        })
        return r['account_secrets']

    def get_txos_for_account(self, account_id, offset=0, limit=100):
        r = self._req({
            "method": "get_txos_for_account",
            "params": {
                "account_id": account_id,
                "offset": str(int(offset)),
                "limit": str(int(limit)),
            }
        })
        return r['txo_map']

    def get_txo(self, txo_id):
        r = self._req({
            "method": "get_txo",
            "params": {
                "txo_id": txo_id,
            },
        })
        return r['txo']

    def get_balance_for_account(self, account_id):
        r = self._req({
            "method": "get_balance_for_account",
            "params": {
                "account_id": account_id,
            }
        })
        return r['balance']

    def get_balance_for_address(self, address):
        r = self._req({
            "method": "get_balance_for_address",
            "params": {
                "address": address,
            }
        })
        return r['balance']

    def assign_address_for_account(self, account_id, metadata=None):
        if metadata is None:
            metadata = ''

        r = self._req({
            "method": "assign_address_for_account",
            "params": {
                "account_id": account_id,
                "metadata": metadata,
            },
        })
        return r['address']

    def get_addresses_for_account(self, account_id, offset=0, limit=100):
        r = self._req({
            "method": "get_addresses_for_account",
            "params": {
                "account_id": account_id,
                "offset": str(int(offset)),
                "limit": str(int(limit)),
            },
        })
        return r['address_map']

    def build_and_submit_transaction(self, account_id, amount, to_address, fee=None):
        params = {
            "account_id": account_id,
            "addresses_and_values": [(to_address, str(amount.value))],
        }
        if fee is not None:
            params['fee'] = str(fee.value)
        r = self._req({
            "method": "build_and_submit_transaction",
            "params": params,
        })
        return r['transaction_log'], r['tx_proposal']

    def build_transaction(
        self,
        account_id,
        addresses_and_amounts,
        tombstone_block=None,
        fee=None,
    ):
        params = {
            "account_id": account_id,
            "addresses_and_values": [],
        }
        for (address, amount) in addresses_and_amounts.items():
            assert amount.token == MOB
            params['addresses_and_values'].append((address, str(amount.value)))
        if fee is not None:
            params['fee_value'] = str(fee.value)
            params['fee_token_id'] = str(fee.token.token_id)
        if tombstone_block is not None:
            params['tombstone_block'] = str(int(tombstone_block))

        r = self._req({
            "method": "build_transaction",
            "params": params,
        })
        return r['tx_proposal'], r['transaction_log_id']

    def submit_transaction(self, tx_proposal, account_id=None):
        r = self._req({
            "method": "submit_transaction",
            "params": {
                "tx_proposal": tx_proposal,
                "account_id": account_id,
            },
        })
        return r['transaction_log']

    def get_transaction_log(self, transaction_log_id):
        r = self._req({
            "method": "get_transaction_log",
            "params": {
                "transaction_log_id": transaction_log_id,
            },
        })
        return r['transaction_log']

    def get_transaction_logs_for_account(self, account_id, offset=0, limit=100):
        r = self._req({
            "method": "get_transaction_logs_for_account",
            "params": {
                "account_id": account_id,
                "offset": str(int(offset)),
                "limit": str(int(limit)),
            },
        })
        return r['transaction_log_map']

    def create_receiver_receipts(self, tx_proposal):
        r = self._req({
            "method": "create_receiver_receipts",
            "params": {
                "tx_proposal": tx_proposal,
            },
        })
        return r['receiver_receipts']

    def check_receiver_receipt_status(self, address, receipt):
        r = self._req({
            "method": "check_receiver_receipt_status",
            "params": {
                "address": address,
                "receiver_receipt": receipt,
            }
        })
        return r

    def build_gift_code(self, account_id, amount, memo=""):
        r = self._req({
            "method": "build_gift_code",
            "params": {
                "account_id": account_id,
                "value_pmob": str(amount.value),
                "memo": memo,
            },
        })
        return r

    def submit_gift_code(self, gift_code_b58, tx_proposal, account_id):
        r = self._req({
            "method": "submit_gift_code",
            "params": {
                "gift_code_b58": gift_code_b58,
                "tx_proposal": tx_proposal,
                "from_account_id": account_id,
            },
        })
        return r['gift_code']

    def get_gift_code(self, gift_code_b58):
        r = self._req({
            "method": "get_gift_code",
            "params": {
                "gift_code_b58": gift_code_b58,
            },
        })
        return r['gift_code']

    def check_gift_code_status(self, gift_code_b58):
        r = self._req({
            "method": "check_gift_code_status",
            "params": {
                "gift_code_b58": gift_code_b58,
            },
        })
        return r

    def get_all_gift_codes(self):
        r = self._req({
            "method": "get_all_gift_codes",
        })
        return r['gift_codes']

    def claim_gift_code(self, account_id, gift_code_b58):
        r = self._req({
            "method": "claim_gift_code",
            "params": {
                "account_id": account_id,
                "gift_code_b58": gift_code_b58,
            },
        })
        return r['txo_id']

    def remove_gift_code(self, gift_code_b58):
        r = self._req({
            "method": "remove_gift_code",
            "params": {
                "gift_code_b58": gift_code_b58,
            },
        })
        return r['removed']

    def create_view_only_account_sync_request(self, account_id):
        r = self._req({
            "method": "create_view_only_account_sync_request",
            "params": {
                "account_id": account_id,
            },
        })
        return r

    def sync_view_only_account(self, params):
        r = self._req({
            "method": "sync_view_only_account",
            "params": params,
        })
        return r

    def version(self):
        r = self._req({"method": "version"})
        return r

    def get_network_status(self):
        r = self._req({
            "method": "get_network_status",
        })
        return r['network_status']

    def get_block(self, block_index):
        r = self._req({
            "method": "get_block",
            "params": {
                "block_index": str(block_index),
            }
        })
        return r['block'], r['block_contents']

    def get_wallet_status(self):
        r = self._req({"method": "get_wallet_status"})
        return r['wallet_status']

    # Utility methods.

    @staticmethod
    def poll(func, delay=1.0, timeout=30):
        """
        Repeatedly call the given function until it returns a result.
        """
        start = time.monotonic()
        while True:
            elapsed = time.monotonic() - start
            if elapsed >= timeout:
                raise TimeoutError('Polling timed out before succeeding.')
            try:
                result = func()
            except ConnectionError:
                result = None
            if result is not None:
                return result
            time.sleep(delay)

    def poll_balance(self, account_id, min_block_height=None, **kwargs):
        """
        Poll the balance endpoint for this account, until it reports as synced.

        If the `min_block_height` argument is given, don't return until it is
        synced above the given block height. This is a way to wait for a
        submitted transaction to complete.
        """
        def func():
            balance = self.get_balance_for_account(account_id)
            if balance['is_synced']:
                if (
                    min_block_height is None
                    or int(balance['account_block_height']) >= min_block_height
                ):
                    return balance
        return self.poll(func, **kwargs)
