# Copyright (c) 2022 MobileCoin, Inc.
import local_network
import time

from test_utils.utilities import parse_network_type_cmd_line_args
from test_utils import fullservice as fslib

# Shoving the raw request into a python class would allow us to just access these things as members of the class
# TODO: create a python dataclass to hold these
BALANCE = 'balance'
PENDING = 'pending_pmob'
UNSPENT = 'unspent_pmob'
SPENT = 'spent_pmob'
FEE = 'fee_pmob'

# run sample test transactions between the first two accounts in full service
def test_transactions_basic(fs):
    if fs.account_ids is None:
        raise Exception(f'accounts not found in wallet')
    elif len(fs.account_ids) < 2:
        raise Exception(f'found {len(fs.account_ids)} account(s), minimum required is 2')

    acc0_id = fs.account_ids[0]
    acc1_id = fs.account_ids[1]    
    
    account_0 = fs.account_map[acc0_id]
    account_1 = fs.account_map[acc1_id]
    p_mob_amount = 600_000_000
    p_mob_amount_str = str(p_mob_amount)

    acc0_balance0 = (fs.get_account_status(acc0_id))[BALANCE]
    acc1_balance0 = (fs.get_account_status(acc1_id))[BALANCE]
    
    assert int(acc0_balance0[UNSPENT]) > p_mob_amount    

    log_0 = fs.send_transaction(acc0_id, account_1['main_address'], p_mob_amount_str, False)
    attempts_count = 0
    while fs.get_account_status(acc0_id)[BALANCE][PENDING] != '0' and attempts_count < 5:
        time.sleep(1)
        attempts_count += 1
    acc0_balance1 = fs.get_account_status(acc0_id)[BALANCE]
    acc1_balance1 = fs.get_account_status(acc1_id)[BALANCE]
    assert acc0_balance1[PENDING] == '0' 
    assert int(acc0_balance1[UNSPENT]) == int(acc0_balance0[UNSPENT]) - p_mob_amount - int(log_0[FEE])
    assert int(acc1_balance1[UNSPENT]) == int(acc1_balance0[UNSPENT]) + p_mob_amount 
    
    log_1 = fs.send_transaction(acc1_id, account_0['main_address'], p_mob_amount_str, False)
    attempts_count = 0
    while fs.get_account_status(acc1_id)[BALANCE][PENDING] != '0' and attempts_count < 5:
        time.sleep(1)
        attempts_count += 1
    acc0_balance2 = fs.get_account_status(acc0_id)[BALANCE]
    acc1_balance2 = fs.get_account_status(acc1_id)[BALANCE]
    assert acc1_balance2[PENDING] == '0' 
    assert int(acc0_balance2[UNSPENT]) == int(acc0_balance1[UNSPENT]) + p_mob_amount
    assert int(acc1_balance2[UNSPENT]) == int(acc1_balance1[UNSPENT]) - p_mob_amount - int(log_1[FEE]) 

    print(('________________________________________________________________________________'))
    print('transactions completed')

if __name__ == '__main__':
    # TODO: This test can probably just always use the same network argument so we don't need it from command line

    args = parse_network_type_cmd_line_args()

    # start networks
    print('________________________________________________________________________________')
    print('Starting networks')
    mobilecoin_network = local_network.Network()
    mobilecoin_network.default_entry_point(args.network_type, args.block_version)
    with fslib.FullService() as fs:
        fs.sync_full_service_to_network(mobilecoin_network)
    
        try:
            print('________________________________________________________________________________')
            print('Importing accounts')
            # import accounts
            fs.setup_accounts()
            wallet_status = fs.get_wallet_status()
    
            # verify accounts have been imported
            for account_id in fs.account_ids:
                account_balance = fs.get_account_status(account_id)
    
            # run test suite
            test_transactions_basic(fs)
            
            # successful exit on no error
            print("Yay test passed!")
        except Exception as e:
            print(e)

    local_network.cleanup(fs, mobilecoin_network)
