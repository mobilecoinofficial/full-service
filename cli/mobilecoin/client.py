from decimal import Decimal
import http.client
import json
import time
from urllib.parse import urlparse

DEFAULT_URL = 'http://127.0.0.1:9090/wallet'

MAX_TOMBSTONE_BLOCKS = 100


class WalletAPIError(Exception):
    def __init__(self, response):
        self.response = response


class Client:

    def __init__(self, url=None, verbose=False):
        if url is None:
            url = DEFAULT_URL
        self.url = url
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

        try:
            response_data = json.load(r)
        except ValueError:
            raise ValueError('API returned invalid JSON:', r.text)

        if self.verbose:
            print(r.status, http.client.responses[r.status])
            print(json.dumps(response_data, indent=2))
            print()

        # Check for errors and unwrap result.
        try:
            result = response_data['result']
        except KeyError:
            raise WalletAPIError(response_data)

        self._query_count += 1

        return result

    def create_account(self, name=None):
        r = self._req({
            "method": "create_account",
            "params": {
                "name": name,
            }
        })
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

    def get_all_accounts(self):
        r = self._req({"method": "get_all_accounts"})
        return r['account_map']

    def get_account(self, account_id):
        r = self._req({
            "method": "get_account",
            "params": {"account_id": account_id}
        })
        return r['account']

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

    def get_all_txos_for_account(self, account_id):
        r = self._req({
            "method": "get_all_txos_for_account",
            "params": {"account_id": account_id}
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

    def get_network_status(self):
        r = self._req({
            "method": "get_network_status",
        })
        return r['network_status']

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

    def get_addresses_for_account(self, account_id, offset=0, limit=1000):
        r = self._req({
            "method": "get_addresses_for_account",
            "params": {
                "account_id": account_id,
                "offset": str(int(offset)),
                "limit": str(int(limit)),
            },
        })
        return r['address_map']

    def _build_and_submit_transaction(self, account_id, amount, to_address, fee):
        amount = str(mob2pmob(amount))
        params = {
            "account_id": account_id,
            "addresses_and_values": [(to_address, amount)],
        }
        if fee is not None:
            params['fee'] = str(mob2pmob(fee))
        r = self._req({
            "method": "build_and_submit_transaction",
            "params": params,
        })
        return r

    def build_and_submit_transaction(self, account_id, amount, to_address, fee=None):
        r = self._build_and_submit_transaction(account_id, amount, to_address, fee)
        return r['transaction_log']

    def build_and_submit_transaction_with_proposal(self, account_id, amount, to_address, fee=None):
        r = self._build_and_submit_transaction(account_id, amount, to_address, fee)
        return r['transaction_log'], r['tx_proposal']

    def build_transaction(self, account_id, amount, to_address, tombstone_block=None, fee=None):
        amount = str(mob2pmob(amount))
        params = {
            "account_id": account_id,
            "addresses_and_values": [(to_address, amount)],
        }
        if tombstone_block is not None:
            params['tombstone_block'] = str(int(tombstone_block))
        if fee is not None:
            params['fee'] = str(mob2pmob(fee))
        r = self._req({
            "method": "build_transaction",
            "params": params,
        })
        return r['tx_proposal']

    def submit_transaction(self, tx_proposal, account_id=None):
        r = self._req({
            "method": "submit_transaction",
            "params": {
                "tx_proposal": tx_proposal,
                "account_id": account_id,
            },
        })
        return r['transaction_log']

    def get_all_transaction_logs_for_account(self, account_id):
        r = self._req({
            "method": "get_all_transaction_logs_for_account",
            "params": {
                "account_id": account_id,
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
        amount = str(mob2pmob(amount))
        r = self._req({
            "method": "build_gift_code",
            "params": {
                "account_id": account_id,
                "value_pmob": amount,
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

    # Utility methods.

    def poll_balance(self, account_id, min_block_height=None, seconds=10):
        for _ in range(seconds):
            balance = self.get_balance_for_account(account_id)
            if balance['is_synced']:
                if (
                    min_block_height is None
                    or int(balance['account_block_height']) >= min_block_height
                ):
                    return balance
            time.sleep(1.0)
        else:
            raise Exception('Could not sync account {}'.format(account_id))

    def poll_gift_code_status(self, gift_code_b58, target_status, seconds=10):
        for _ in range(seconds):
            response = self.check_gift_code_status(gift_code_b58)
            if response['gift_code_status'] == target_status:
                return response
            time.sleep(1.0)
        else:
            raise Exception('Gift code {} never reached status {}.'.format(gift_code_b58, target_status))

    def poll_txo(self, txo_id, seconds=10):
        for _ in range(10):
            try:
                return self.get_txo(txo_id)
            except WalletAPIError:
                pass
            time.sleep(1)
        else:
            raise Exception('Txo {} never landed.'.format(txo_id))


PMOB = Decimal("1e12")


def mob2pmob(x):
    """Convert from MOB to picoMOB."""
    result = round(Decimal(x) * PMOB)
    assert 0 <= result < 2**64
    return result


def pmob2mob(x):
    """Convert from picoMOB to MOB."""
    result = int(x) / PMOB
    if result == 0:
        result = Decimal("0")
    return result
