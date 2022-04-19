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

pub struct SeedGiftCodesResult {
    unsubmitted: EncodedGiftCode,
    submitted: EncodedGiftCode,
    claimed: EncodedGiftCode,
}
pub fn seed_gift_codes(
    conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    ledger_db: &mut LedgerDB,
    wallet_db: &WalletDb,
    service: &WalletService<MockBlockchainConnection<LedgerDB>, MockFogPubkeyResolver>,
    logger: &Logger,
    account: &Account,
    receiver_account: &Account,
) -> SeedGiftCodesResult {
    let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

    // Add a block with a transaction for the gifter account
    let gifter_account_key: AccountKey = mc_util_serial::decode(&account.account_key).unwrap();
    let gifter_public_address =
        &gifter_account_key.subaddress(account.main_subaddress_index as u64);
    let gifter_account_id = AccountID(account.account_id_hex.to_string());

    add_block_to_ledger_db(
        ledger_db,
        &vec![gifter_public_address.clone()],
        100 * MOB as u64,
        &vec![KeyImage::from(rng.next_u64())],
        &mut rng,
    );
    manually_sync_account(ledger_db, wallet_db, &gifter_account_id, logger);

    // Create 3 gift codes
    let (tx_proposal, gift_code_b58) = service
        .build_gift_code(
            &gifter_account_id,
            2 * MOB as u64,
            Some("Gift code".to_string()),
            None,
            None,
            None,
            None,
        )
        .unwrap();

    // going to submit but not claim this code
    let gift_code_1_submitted = service
        .submit_gift_code(
            &gifter_account_id,
            &gift_code_b58.clone(),
            &tx_proposal.clone(),
        )
        .unwrap();

    add_block_with_tx_proposal(ledger_db, tx_proposal);
    manually_sync_account(&ledger_db, &service.wallet_db, &gifter_account_id, &logger);

    let (tx_proposal, gift_code_b58) = service
        .build_gift_code(
            &gifter_account_id,
            2 * MOB as u64,
            Some("Gift code".to_string()),
            None,
            None,
            None,
            None,
        )
        .unwrap();

    // going to submit and claim this one
    let gift_code_2_claimed = service
        .submit_gift_code(
            &gifter_account_id,
            &gift_code_b58.clone(),
            &tx_proposal.clone(),
        )
        .unwrap();

    add_block_with_tx_proposal(ledger_db, tx_proposal);
    manually_sync_account(&ledger_db, &service.wallet_db, &gifter_account_id, &logger);

    // leave this code as pending
    let (tx_proposal, gift_code_b58_pending) = service
        .build_gift_code(
            &gifter_account_id,
            2 * MOB as u64,
            Some("Gift code".to_string()),
            None,
            None,
            None,
            None,
        )
        .unwrap();

    // Claim the gift code to another account
    manually_sync_account(
        &ledger_db,
        &service.wallet_db,
        &AccountID(receiver_account.account_id_hex.clone()),
        &logger,
    );

    let tx = service
        .claim_gift_code(
            &EncodedGiftCode(gift_code_2_claimed.gift_code_b58.clone()),
            &AccountID(receiver_account.account_id_hex.clone()),
            None,
        )
        .unwrap();
    add_block_with_tx(ledger_db, tx);
    manually_sync_account(
        &ledger_db,
        &service.wallet_db,
        &AccountID(receiver_account.account_id_hex.clone()),
        &logger,
    );

    SeedGiftCodesResult {
        unsubmitted: gift_code_b58_pending,
        submitted: EncodedGiftCode(gift_code_1_submitted.gift_code_b58),
        claimed: EncodedGiftCode(gift_code_2_claimed.gift_code_b58),
    }
}

pub fn test_gift_codes(
    gift_codes: &SeedGiftCodesResult,
    service: &WalletService<MockBlockchainConnection<LedgerDB>, MockFogPubkeyResolver>,
) {
    let (status, _gift_code_value_opt, _memo) = service
        .check_gift_code_status(&gift_codes.unsubmitted)
        .unwrap();
    assert_eq!(status, GiftCodeStatus::GiftCodeSubmittedPending);

    let (status, _gift_code_value_opt, _memo) = service
        .check_gift_code_status(&gift_codes.submitted)
        .unwrap();
    assert_eq!(status, GiftCodeStatus::GiftCodeAvailable);

    let (status, _gift_code_value_opt, _memo) =
        service.check_gift_code_status(&gift_codes.claimed).unwrap();
    assert_eq!(status, GiftCodeStatus::GiftCodeClaimed);
}
