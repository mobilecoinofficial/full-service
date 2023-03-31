import pytest

from mobilecoin.client import (
    MirrorClientAsync,
)


@pytest.fixture(scope='session')
async def mirror_client():
    async with MirrorClientAsync() as mirror_client:
        yield mirror_client


async def test_illegal_methods(mirror_client):
    methods = [
        'assign_address_for_account',
        'build_and_submit_transaction',
        'build_gift_code',
        'build_split_txo_transaction',
        'build_transaction',
        'check_b58_type',
        'check_gift_code_status',
        'claim_gift_code',
        'create_account',
        'create_receiver_receipts',
        'export_account_secrets',
        'get_all_addresses_for_account',
        'get_all_gift_codes',
        'get_all_transaction_logs_for_account',
        'get_all_transaction_logs_ordered_by_block',
        'get_all_txos_for_account',
        'get_all_txos_for_address',
        'get_gift_code',
        'get_mc_protocol_transaction',
        'get_mc_protocol_txo',
        'get_txo',
        'get_txos_for_account',
        'import_account',
        'import_account_from_legacy_root_entropy',
        'remove_account',
        'remove_gift_code',
        'submit_gift_code',
        'submit_transaction',
        'update_account_name',
    ]
    for method in methods:
        await mirror_client._req({"method": method, "params": {}})

