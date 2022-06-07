use crate::{
    db::{
        account::AccountID,
        models::{Account, TransactionLog, Txo},
        transaction_log::TransactionLogModel,
        txo::TxoModel,
        WalletDb,
    },
    test_utils::{
        add_block_with_db_txos, add_block_with_tx_outs, create_test_minted_and_change_txos,
        create_test_txo_for_recipient, manually_sync_account, MOB,
    },
};
use diesel::{
    r2d2::{ConnectionManager, PooledConnection},
    SqliteConnection,
};
use mc_common::logger::Logger;
use mc_crypto_rand::RngCore;
use mc_ledger_db::LedgerDB;
use mc_transaction_core::{ring_signature::KeyImage, tokens::Mob, Amount, Token};
use rand::{rngs::StdRng, SeedableRng};

// create 1 spent, 1 change (minted), and 1 orphaned txo
pub fn seed_txos(
    _conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ledger_db: &mut LedgerDB,
    wallet_db: &WalletDb,
    logger: &Logger,
    account: &Account,
) {
    let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
    // Create received txo for account
    let account_key = mc_util_serial::decode(&account.account_key).unwrap();
    let (for_account_txo, for_account_key_image) =
        create_test_txo_for_recipient(&account_key, 0, Amount::new(1000 * MOB, Mob::ID), &mut rng);

    // add this txo to the ledger
    add_block_with_tx_outs(
        ledger_db,
        &[for_account_txo.clone()],
        &[KeyImage::from(rng.next_u64())],
    );

    manually_sync_account(
        &ledger_db,
        &wallet_db,
        &AccountID::from(&account_key),
        &logger,
    );

    // "spend" the TXO by sending it to same account, but at a subaddress we
    // have not yet assigned. At the DB layer, we accomplish this by
    // constructing the output txos, then logging sent and received for this
    // account.
    let ((output_txo_id, _output_value), (change_txo_id, _change_value)) =
        create_test_minted_and_change_txos(
            account_key.clone(),
            account_key.subaddress(4),
            33 * MOB,
            wallet_db.clone(),
            ledger_db.clone(),
            logger.clone(),
        );

    add_block_with_db_txos(
        ledger_db,
        &wallet_db,
        &[output_txo_id, change_txo_id],
        &[KeyImage::from(for_account_key_image)],
    );

    manually_sync_account(
        &ledger_db,
        &wallet_db,
        &AccountID::from(&account_key),
        &logger,
    );
}

pub fn test_txos(
    account_id: AccountID,
    conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
) {
    // validate expected txo states
    let txos = Txo::list_for_account(&account_id.to_string(), None, None, Some(0), &conn).unwrap();
    assert_eq!(txos.len(), 3);

    // Check that we have 2 spendable (1 is orphaned)
    let spendable: Vec<&Txo> = txos.iter().filter(|f| f.key_image.is_some()).collect();
    assert_eq!(spendable.len(), 2);

    // Check that we have one spent - went from [Received, Unspent] -> [Received,
    // Spent]
    let spent = Txo::list_spent(&account_id.to_string(), None, Some(0), &conn).unwrap();
    assert_eq!(spent.len(), 1);
    assert_eq!(spent[0].spent_block_index.clone().unwrap(), 13);
    assert_eq!(spent[0].minted_account_id_hex, None);

    // Check that we have one orphaned - went from [Minted, Secreted] -> [Minted,
    // Orphaned]
    let orphaned = Txo::list_orphaned(&account_id.to_string(), Some(0), &conn).unwrap();
    assert_eq!(orphaned.len(), 1);
    assert!(orphaned[0].key_image.is_none());
    assert_eq!(orphaned[0].received_block_index.clone().unwrap(), 13);
    assert!(orphaned[0].minted_account_id_hex.is_some());
    assert!(orphaned[0].received_account_id_hex.is_some());

    // Check that we have one unspent (change) - went from [Minted, Secreted] ->
    // [Minted, Unspent]
    let unspent = Txo::list_unspent(&account_id.to_string(), None, Some(0), &conn).unwrap();
    assert_eq!(unspent.len(), 1);
    assert_eq!(unspent[0].received_block_index.clone().unwrap(), 13);

    // Check that a transaction log entry was created for each received TxOut (note:
    // we are not creating submit logs in this test)
    let transaction_logs =
        TransactionLog::list_all(&account_id.to_string(), None, None, &conn).unwrap();
    assert_eq!(transaction_logs.len(), 3);
}
