import local_network
import time

from test_utils.utilities import parse_network_type_cmd_line_args
from test_utils import fullservice as fslib

# run sample test transactions between the first two accounts in full service
def test_transactions(fs):
    print(('________________________________________________________________________________'))
    print('testing transaction sends')
    if fs.account_ids is None:
        print(f'accounts not found in wallet')
        local_network.cleanup(1)
    elif len(fs.account_ids) < 2:
        print(f'found {len(fs.account_ids)} account(s), minimum required is 2')
        local_network.cleanup(1)
    account_0 = fs.account_map[fs.account_ids[0]]
    account_1 = fs.account_map[fs.account_ids[1]]
    p_mob_amount = str(600_000_000)

    # flakey tests due to accounts having a variable amount of pmob. This needs to be controlled for use.
    log_0 = fs.send_transaction(account_0['account_id'], account_1['main_address'], p_mob_amount)
    log_1 = fs.send_transaction(account_1['account_id'], account_0['main_address'], p_mob_amount)

    print(('________________________________________________________________________________'))
    print('transactions completed')
    print(f'transaction 0 log: {log_0}')
    print(f'transaction 1 log: {log_1}')

if __name__ == '__main__':
    # TODO: This test can probably just always use the same network argument so we don't need it from command line

    args = parse_network_type_cmd_line_args()

    # start networks
    print('________________________________________________________________________________')
    print('Starting networks')
    mobilecoin_network = None
    mobilecoin_network = local_network.Network()
    mobilecoin_network.default_entry_point(args.network_type, args.block_version)
    fs = fslib.FullService()
    with fs as full_service:
        #full_service = fslib.FullService()
        print(type(full_service))
        full_service.sync_full_service_to_network(mobilecoin_network)
    
        try:
            print('________________________________________________________________________________')
            print('Importing accounts')
            # import accounts
            full_service.setup_accounts()
            wallet_status = full_service.get_wallet_status()
    
            # verify accounts have been imported, view initial account state
            for account_id in full_service.account_ids:
                balance = full_service.get_account_status(account_id)
                print(f'account_id {account_id} : balance {balance}')
    
            # run test suite
            test_transactions(full_service)
    
            # allow for transactions to pass through
            # flakey -- replace with checker function
            time.sleep(20)
    
            # verify accounts have been updated with changed state
            # TODO: bundle with test suite, exiting code 0 on success, or code 1 on failure
            for account_id in full_service.account_ids:
                print(account_id)
                balance = full_service.get_account_status(account_id)['balance']
                print(f'account_id {account_id} : balance {balance}')
            
            # successful exit on no error
            local_network.cleanup(full_service, mobilecoin_network)
            print("Yay test passed!")
        except Exception as e:
            print(e)
            local_network.cleanup(full_service, mobilecoin_network)
