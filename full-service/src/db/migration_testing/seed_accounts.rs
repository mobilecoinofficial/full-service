use crate::{
    db::{
        account::AccountID,
        models::{Account, TransactionLog, Txo},
        transaction_log::TransactionLogModel,
        txo::{TxoID, TxoModel},
        WalletDb,
    },
    service::{
        account::AccountService,
        gift_code::{EncodedGiftCode, GiftCodeService, GiftCodeStatus},
        WalletService,
    },
    test_utils::{
        add_block_to_ledger_db, add_block_with_db_txos, add_block_with_tx, add_block_with_tx_outs,
        add_block_with_tx_proposal, create_test_minted_and_change_txos, create_test_received_txo,
        create_test_txo_for_recipient, get_resolver_factory, get_test_ledger,
        manually_sync_account, random_account_with_seed_values, WalletDbTestContext, MOB,
    },
};
use diesel::{
    r2d2::{ConnectionManager, PooledConnection},
    SqliteConnection,
};
use mc_account_keys::AccountKey;
use mc_common::logger::Logger;
use mc_connection_test_utils::MockBlockchainConnection;
use mc_crypto_rand::RngCore;
use mc_fog_report_validation::MockFogPubkeyResolver;
use mc_ledger_db::LedgerDB;
use mc_transaction_core::{
    encrypted_fog_hint::EncryptedFogHint,
    onetime_keys::{create_tx_out_target_key, recover_onetime_private_key},
    ring_signature::KeyImage,
    tx::{Tx, TxOut},
    Block, BlockContents, BLOCK_VERSION,
};
use rand::{rngs::StdRng, SeedableRng};
use std::collections::HashMap;

pub fn seed_accounts(
    service: &WalletService<MockBlockchainConnection<LedgerDB>, MockFogPubkeyResolver>,
) -> (Account, Account) {
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

    (txo_account, gift_code_account)
}
