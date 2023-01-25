import sys
import time

from mobilecoin import Client
from mobilecoin.cli import _load_import

c = Client(verbose=False)

source_wallet = sys.argv[1]
data = _load_import(source_wallet)
source_account = c.import_account(**data)
source_account_id = source_account['account_id']

start = time.monotonic()
balance = c.poll_balance(source_account_id, seconds=600, poll_delay=0.2)
end = time.monotonic()

c.remove_account(source_account_id)

print(round(end - start, 1), 'seconds')
print(int(balance['unspent_pmob']) / 1e12, 'MOB')
