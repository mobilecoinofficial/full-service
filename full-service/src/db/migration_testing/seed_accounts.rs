use crate::{
    db::models::Account,
    service::{account::AccountService, WalletService},
};
use mc_connection_test_utils::MockBlockchainConnection;
use mc_fog_report_validation::MockFogPubkeyResolver;
use mc_ledger_db::LedgerDB;

pub fn seed_accounts(
    service: &WalletService<MockBlockchainConnection<LedgerDB>, MockFogPubkeyResolver>,
) -> (Account, Account, Account) {
    let txo_account = service
        .create_account(
            Some("txo_account".to_string()),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        )
        .unwrap();

    let gift_code_account = service
        .create_account(
            Some("gift_code_account".to_string()),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        )
        .unwrap();

    let gift_code_receiver_account = service
        .create_account(
            Some("gift_code_receiver_account".to_string()),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        )
        .unwrap();

    (txo_account, gift_code_account, gift_code_receiver_account)
}

pub fn test_accounts(
    service: &WalletService<MockBlockchainConnection<LedgerDB>, MockFogPubkeyResolver>,
) {
    let accounts = service.list_accounts().unwrap();

    assert_eq!(accounts.len(), 3);
}
