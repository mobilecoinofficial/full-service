import requests

FS_URL = 'http://localhost:9090/wallet/v2'

def fs_request(method, params=None):
    json = {"method": method, "jsonrpc":"2.0", "id":1}
    if params:
        json['params'] = params
    return requests.post(FS_URL, json={"method": method, "jsonrpc":"2.0", "id":1, "params": params}).json()

def get_block_height():
    response = fs_request('get_network_status')
    return response['result']['network_status']['network_block_height']

def build_transaction(block):
    params = {"account_id": "353a142530426bed601805ef0da8412dd323797bd07614202b8acdd0a249a1c5",
              "recipient_public_address": "2hQXjtgFSQ7XjAu1JYtBwUjJDZEBFr94FQv6f8ELDBr9BU2ngoug6WoM36naNMwsx87mkuw97xCCysTcRKQ5q5DNfzT5hQ3V724m57Mzx42",
              "amount": {"value": "100000", "token_id": "1"},
              "tombstone_block": str(block)
              }
    response = fs_request('build_transaction', params)
    return response['result']['tx_proposal']

def submit_transaction(transaction):
    params = {
        "account_id": "353a142530426bed601805ef0da8412dd323797bd07614202b8acdd0a249a1c5",
        "tx_proposal": transaction
    }
    response = fs_request('submit_transaction', params)
    return response

def main():
    while True:
        block = int(get_block_height())
        transaction = build_transaction(block + 1)
        response = submit_transaction(transaction)
        print(f"the response is {response}")

if __name__ == '__main__':
    main()
