// Copyright (c) 2020 MobileCoin Inc.

use crate::{
    db::WalletDb,
    db_models::{account::AccountID, txo::TxoModel},
    models::Txo,
    transaction_builder::WalletTransactionBuilder,
};
use diesel::{prelude::*, SqliteConnection};
use diesel_migrations::embed_migrations;
use mc_account_keys::{AccountKey, PublicAddress};
use mc_attest_core::Verifier;
use mc_common::logger::Logger;
use mc_connection::{ConnectionManager, ThickClient};
use mc_connection_test_utils::{test_client_uri, MockBlockchainConnection};
use mc_crypto_keys::RistrettoPrivate;
use mc_crypto_rand::{CryptoRng, RngCore};
use mc_fog_report_validation::MockFogPubkeyResolver;
use mc_ledger_db::{Ledger, LedgerDB};
use mc_transaction_core::{
    encrypted_fog_hint::EncryptedFogHint, ring_signature::KeyImage, tx::TxOut,
};
use mc_transaction_core::{Block, BlockContents, BLOCK_VERSION};
use mc_util_from_random::FromRandom;
use rand::rngs::StdRng;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::path::PathBuf;
use std::sync::Arc;
use tempdir::TempDir;

embed_migrations!("migrations/");

pub const MOB: i64 = 1_000_000_000_000;

/// The amount each recipient gets in the test ledger.
pub const DEFAULT_PER_RECIPIENT_AMOUNT: u64 = 5_000 * 1_000_000_000_000;

pub struct WalletDbTestContext {
    base_url: String,
    pub db_name: String,
}

impl Default for WalletDbTestContext {
    fn default() -> Self {
        dotenv::dotenv().unwrap();

        let db_name: String = format!(
            "test_{}",
            thread_rng()
                .sample_iter(&Alphanumeric)
                .take(10)
                .collect::<String>()
                .to_lowercase()
        );
        let base_url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");

        // Connect to the database and run the migrations
        let conn = SqliteConnection::establish(&format!("{}/{}", base_url, db_name))
            .unwrap_or_else(|err| panic!("Cannot connect to {} database: {:?}", db_name, err));

        embedded_migrations::run(&conn).expect("failed running migrations");

        // Success
        Self { base_url, db_name }
    }
}

impl WalletDbTestContext {
    pub fn get_db_instance(&self, logger: Logger) -> WalletDb {
        WalletDb::new_from_url(&format!("{}/{}", self.base_url, self.db_name), logger)
            .expect("failed creating new SqlRecoveryDb")
    }
}

/// Sets up ledger_db. Each block contains one txo per recipient.
///
/// # Arguments
/// *
/// * `num_random_recipients` - Number of random recipients to create.
/// * `known_recipients` - A list of known recipients to create.
/// * `num_blocks` - Number of blocks to create in the ledger_db.
/// * `rng`
///
/// Note that all txos will be controlled by the subindex DEFAULT_SUBADDRESS_INDEX
pub fn get_test_ledger(
    num_random_recipients: u32,
    known_recipients: &[PublicAddress],
    num_blocks: usize,
    mut rng: &mut (impl CryptoRng + RngCore),
) -> LedgerDB {
    let mut public_addresses: Vec<PublicAddress> = (0..num_random_recipients)
        .map(|_i| mc_account_keys::AccountKey::random(&mut rng).default_subaddress())
        .collect();

    public_addresses.extend(known_recipients.iter().cloned());

    // Note that TempDir manages uniqueness by constructing paths
    // like: /tmp/ledger_db.tvF0XHTKsilx
    let ledger_db_tmp = TempDir::new("ledger_db").expect("Could not make tempdir for ledger db");
    let ledger_db_path = ledger_db_tmp
        .path()
        .to_str()
        .expect("Could not get path as string");

    let mut ledger_db = generate_ledger_db(&ledger_db_path);

    for block_index in 0..num_blocks {
        let key_images = if block_index == 0 {
            vec![]
        } else {
            vec![KeyImage::from(rng.next_u64())]
        };
        let _new_block_height = add_block_to_ledger_db(
            &mut ledger_db,
            &public_addresses,
            DEFAULT_PER_RECIPIENT_AMOUNT,
            &key_images,
            rng,
        );
    }

    ledger_db
}

/// Creates an empty LedgerDB.
///
/// # Arguments
/// * `path` - Path to the ledger's data.mdb file. If such a file exists, it will be replaced.
fn generate_ledger_db(path: &str) -> LedgerDB {
    // DELETE the old database if it already exists.
    let _ = std::fs::remove_file(format!("{}/data.mdb", path));
    LedgerDB::create(PathBuf::from(path)).expect("Could not create ledger_db");
    let db = LedgerDB::open(PathBuf::from(path)).expect("Could not open ledger_db");
    db
}

/// Adds a block containing one txo for each provided recipient and returns new block height.
///
/// # Arguments
/// * `ledger_db` - Ledger database instance.
/// * `recipients` - Recipients of outputs.
/// * `output_value` - The amount each recipient will get.
/// * `key_images` - Key images to include in the block.
/// * `rng` - Random number generator.
pub fn add_block_to_ledger_db(
    ledger_db: &mut LedgerDB,
    recipients: &[PublicAddress],
    output_value: u64,
    key_images: &[KeyImage],
    rng: &mut (impl CryptoRng + RngCore),
) -> u64 {
    let outputs: Vec<_> = recipients
        .iter()
        .map(|recipient| {
            TxOut::new(
                // TODO: allow for subaddress index!
                output_value,
                recipient,
                &RistrettoPrivate::from_random(rng),
                Default::default(),
            )
            .unwrap()
        })
        .collect();

    let block_contents = BlockContents::new(key_images.to_vec(), outputs.clone());

    let num_blocks = ledger_db.num_blocks().expect("failed to get block height");

    let new_block;
    if num_blocks > 0 {
        let parent = ledger_db
            .get_block(num_blocks - 1)
            .expect("failed to get parent block");
        new_block =
            Block::new_with_parent(BLOCK_VERSION, &parent, &Default::default(), &block_contents);
    } else {
        new_block = Block::new_origin_block(&outputs);
    }

    ledger_db
        .append_block(&new_block, &block_contents, None)
        .expect("failed writing initial transactions");

    ledger_db.num_blocks().expect("failed to get block height")
}

pub fn setup_peer_manager(
    ledger_db: LedgerDB,
    logger: Logger,
) -> ConnectionManager<MockBlockchainConnection<LedgerDB>> {
    let peer1 = MockBlockchainConnection::new(test_client_uri(1), ledger_db.clone(), 0);
    let peer2 = MockBlockchainConnection::new(test_client_uri(2), ledger_db.clone(), 0);

    ConnectionManager::new(vec![peer1, peer2], logger.clone())
}

pub fn setup_grpc_peer_manager(logger: Logger) -> ConnectionManager<ThickClient> {
    let peer1 = test_client_uri(1);
    let peer2 = test_client_uri(2);
    let peers = vec![peer1, peer2];

    let grpc_env = Arc::new(
        grpcio::EnvBuilder::new()
            .cq_count(1)
            .name_prefix("peer")
            .build(),
    );

    let verifier = Verifier::default();

    let connected_peers = peers
        .iter()
        .map(|client_uri| {
            ThickClient::new(
                client_uri.clone(),
                verifier.clone(),
                grpc_env.clone(),
                logger.clone(),
            )
            .expect("Could not create thick client.")
        })
        .collect();

    ConnectionManager::new(connected_peers, logger.clone())
}

pub fn create_test_received_txo(
    account_key: &AccountKey,
    recipient_subaddress_index: u64,
    value: u64,
    received_block_height: u64,
    rng: &mut StdRng,
    wallet_db: &WalletDb,
) -> (String, TxOut, KeyImage) {
    let recipient = account_key.subaddress(recipient_subaddress_index);
    let tx_private_key = RistrettoPrivate::from_random(rng);
    let hint = EncryptedFogHint::fake_onetime_hint(rng);
    let txo = TxOut::new(value, &recipient, &tx_private_key, hint).unwrap();

    // Get KeyImage from the onetime private key
    let key_image = KeyImage::from(&tx_private_key);

    let txo_id_hex = Txo::create_received(
        txo.clone(),
        Some(recipient_subaddress_index as i64),
        Some(key_image),
        value,
        received_block_height as i64,
        &AccountID::from(account_key).to_string(),
        &wallet_db.get_conn().unwrap(),
    )
    .unwrap();
    (txo_id_hex, txo, key_image)
}

pub fn create_test_minted_and_change_txos(
    src_account_key: AccountKey,
    recipient: PublicAddress,
    value: u64,
    wallet_db: WalletDb,
    ledger_db: LedgerDB,
    logger: Logger,
) -> (Option<PublicAddress>, String, i64, String) {
    // Use the builder to create valid TxOuts for this account
    let mut builder = WalletTransactionBuilder::<MockFogPubkeyResolver>::new(
        AccountID::from(&src_account_key).to_string(),
        wallet_db.clone(),
        ledger_db,
        None,
        logger,
    );

    builder.add_recipient(recipient, value).unwrap();
    builder.select_txos(None).unwrap();
    builder.set_tombstone(0).unwrap();
    let tx_proposal = builder.build().unwrap();

    // There should be 2 outputs, one to dest and one change
    assert_eq!(tx_proposal.tx.prefix.outputs.len(), 2);

    // Take the first one (we only construct with one outlay currently, could modify build protocol)
    assert_eq!(tx_proposal.outlay_index_to_tx_out_index.len(), 1);
    let outlay_txo_index = tx_proposal.outlay_index_to_tx_out_index[&0];

    let tx_out = tx_proposal.tx.prefix.outputs[outlay_txo_index].clone();

    let res = Txo::create_minted(
        Some(&AccountID::from(&src_account_key).to_string()),
        &tx_out,
        &tx_proposal,
        outlay_txo_index,
        &wallet_db.get_conn().unwrap(),
    );

    res.unwrap()
}
