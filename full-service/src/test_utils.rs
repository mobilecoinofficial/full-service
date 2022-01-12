// Copyright (c) 2020-2021 MobileCoin Inc.

use crate::{
    db::{
        account::{AccountID, AccountModel},
        models::{Account, TransactionLog, Txo, TXO_USED_AS_CHANGE, TXO_USED_AS_OUTPUT},
        transaction_log::TransactionLogModel,
        txo::TxoModel,
        WalletDb, WalletDbError,
    },
    error::SyncError,
    service::{sync::sync_account, transaction_builder::WalletTransactionBuilder},
    WalletService,
};
use diesel::{
    r2d2::{ConnectionManager as CM, PooledConnection},
    Connection as DSLConnection, SqliteConnection,
};
use diesel_migrations::embed_migrations;
use mc_account_keys::{AccountKey, PublicAddress, RootIdentity};
use mc_attest_verifier::Verifier;
use mc_common::logger::{log, Logger};
use mc_connection::{Connection, ConnectionManager, HardcodedCredentialsProvider, ThickClient};
use mc_connection_test_utils::{test_client_uri, MockBlockchainConnection};
use mc_consensus_scp::QuorumSet;
use mc_crypto_keys::{RistrettoPrivate, RistrettoPublic};
use mc_crypto_rand::{CryptoRng, RngCore};
use mc_fog_report_validation::{FullyValidatedFogPubkey, MockFogPubkeyResolver};
use mc_ledger_db::{Ledger, LedgerDB};
use mc_ledger_sync::PollingNetworkState;
use mc_mobilecoind::payments::TxProposal;
use mc_transaction_core::{
    encrypted_fog_hint::EncryptedFogHint,
    onetime_keys::{create_tx_out_target_key, recover_onetime_private_key},
    ring_signature::KeyImage,
    tx::{Tx, TxOut},
    Block, BlockContents, BLOCK_VERSION,
};
use mc_util_from_random::FromRandom;
use mc_util_uri::{ConnectionUri, FogUri};
use rand::{distributions::Alphanumeric, rngs::StdRng, thread_rng, Rng, SeedableRng};
use std::{
    convert::TryFrom,
    env,
    path::PathBuf,
    sync::{Arc, RwLock},
    time::Duration,
};
use tempdir::TempDir;

embed_migrations!("migrations/");

pub const MOB: i64 = 1_000_000_000_000;

/// The amount each recipient gets in the test ledger.
pub const DEFAULT_PER_RECIPIENT_AMOUNT: u64 = 5_000 * MOB as u64;

pub struct WalletDbTestContext {
    base_url: String,
    pub db_name: String,
}

impl Default for WalletDbTestContext {
    fn default() -> Self {
        let db_name: String = format!(
            "test_{}",
            thread_rng()
                .sample_iter(&Alphanumeric)
                .take(10)
                .map(char::from)
                .collect::<String>()
                .to_lowercase()
        );
        let base_url = String::from("/tmp");

        // Connect to the database and run the migrations
        // Note: This should be kept in sync wth how the migrations are run in main.rs
        // so as to have faithful tests.
        // Clear environment variables for db encryption.
        env::set_var("MC_PASSWORD", "".to_string());
        env::set_var("MC_CHANGE_PASSWORD", "".to_string());
        let conn = SqliteConnection::establish(&format!("{}/{}", base_url, db_name))
            .unwrap_or_else(|err| panic!("Cannot connect to {} database: {:?}", db_name, err));
        embedded_migrations::run(&conn).expect("failed running migrations");

        // Success
        Self { base_url, db_name }
    }
}

impl WalletDbTestContext {
    pub fn get_db_instance(&self, logger: Logger) -> WalletDb {
        // Note: Setting db_connections too high results in IO Error: Too many open
        // files.
        WalletDb::new_from_url(&format!("{}/{}", self.base_url, self.db_name), 7, logger)
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
/// Note that all txos will be controlled by the subindex
/// DEFAULT_SUBADDRESS_INDEX
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
        let _new_block_index = add_block_to_ledger_db(
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
/// * `path` - Path to the ledger's data.mdb file. If such a file exists, it
///   will be replaced.
pub fn generate_ledger_db(path: &str) -> LedgerDB {
    // DELETE the old database if it already exists.
    let _ = std::fs::remove_file(format!("{}/data.mdb", path));
    LedgerDB::create(&PathBuf::from(path)).expect("Could not create ledger_db");
    let db = LedgerDB::open(&PathBuf::from(path)).expect("Could not open ledger_db");
    db
}

fn append_test_block(ledger_db: &mut LedgerDB, block_contents: BlockContents) -> u64 {
    let num_blocks = ledger_db.num_blocks().expect("failed to get block height");

    let new_block;
    if num_blocks > 0 {
        let parent = ledger_db
            .get_block(num_blocks - 1)
            .expect("failed to get parent block");
        new_block =
            Block::new_with_parent(BLOCK_VERSION, &parent, &Default::default(), &block_contents);
    } else {
        new_block = Block::new_origin_block(&block_contents.outputs);
    }

    ledger_db
        .append_block(&new_block, &block_contents, None)
        .expect("failed writing initial transactions");

    ledger_db.num_blocks().expect("failed to get block height")
}

/// Adds a block containing one txo for each provided recipient and returns new
/// block height.
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
    append_test_block(ledger_db, block_contents)
}

pub fn add_block_with_tx_proposal(ledger_db: &mut LedgerDB, tx_proposal: TxProposal) -> u64 {
    let block_contents = BlockContents::new(
        tx_proposal.tx.key_images(),
        tx_proposal.tx.prefix.outputs.clone(),
    );
    append_test_block(ledger_db, block_contents)
}

pub fn add_block_with_tx(ledger_db: &mut LedgerDB, tx: Tx) -> u64 {
    let block_contents = BlockContents::new(tx.key_images(), tx.prefix.outputs.clone());
    append_test_block(ledger_db, block_contents)
}

pub fn add_block_from_transaction_log(
    ledger_db: &mut LedgerDB,
    conn: &PooledConnection<CM<SqliteConnection>>,
    transaction_log: &TransactionLog,
) -> u64 {
    let associated_txos = transaction_log.get_associated_txos(conn).unwrap();

    let mut output_txos = associated_txos.outputs.clone();
    output_txos.append(&mut associated_txos.change.clone());
    let outputs: Vec<TxOut> = output_txos
        .iter()
        .map(|txo| mc_util_serial::decode(&txo.txo).unwrap())
        .collect();

    let input_txos: Vec<Txo> = associated_txos.inputs.clone();
    let key_images: Vec<KeyImage> = input_txos
        .iter()
        .map(|txo| mc_util_serial::decode(&txo.key_image.clone().unwrap()).unwrap())
        .collect();

    // Note: This block doesn't contain the fee output.

    let block_contents = BlockContents::new(key_images, outputs.clone());

    append_test_block(ledger_db, block_contents)
}

pub fn add_block_with_tx_outs(
    ledger_db: &mut LedgerDB,
    outputs: &[TxOut],
    key_images: &[KeyImage],
) -> u64 {
    let block_contents = BlockContents::new(key_images.to_vec(), outputs.to_vec());
    append_test_block(ledger_db, block_contents)
}

pub fn setup_peer_manager_and_network_state(
    ledger_db: LedgerDB,
    logger: Logger,
) -> (
    ConnectionManager<MockBlockchainConnection<LedgerDB>>,
    Arc<RwLock<PollingNetworkState<MockBlockchainConnection<LedgerDB>>>>,
) {
    let peer1 = MockBlockchainConnection::new(test_client_uri(1), ledger_db.clone(), 0);
    let peer2 = MockBlockchainConnection::new(test_client_uri(2), ledger_db.clone(), 0);

    let peer_manager = ConnectionManager::new(vec![peer1.clone(), peer2.clone()], logger.clone());

    let quorum_set = QuorumSet::new_with_node_ids(
        2,
        vec![
            peer1.uri().responder_id().unwrap(),
            peer2.uri().responder_id().unwrap(),
        ],
    );
    let network_state = Arc::new(RwLock::new(PollingNetworkState::new(
        quorum_set,
        peer_manager.clone(),
        logger.clone(),
    )));

    {
        let mut network_state = network_state.write().unwrap();
        network_state.poll();
    }

    (peer_manager, network_state)
}

pub fn add_block_with_db_txos(
    ledger_db: &mut LedgerDB,
    wallet_db: &WalletDb,
    output_txo_ids: &[String],
    key_images: &[KeyImage],
) -> u64 {
    let outputs: Vec<TxOut> = output_txo_ids
        .iter()
        .map(|txo_id_hex| {
            mc_util_serial::decode(
                &Txo::get(&txo_id_hex.to_string(), &wallet_db.get_conn().unwrap())
                    .unwrap()
                    .txo,
            )
            .unwrap()
        })
        .collect();

    add_block_with_tx_outs(ledger_db, &outputs, key_images)
}

pub fn manually_sync_account(
    ledger_db: &LedgerDB,
    wallet_db: &WalletDb,
    account_id: &AccountID,
    target_block_index: u64,
    logger: &Logger,
) -> Account {
    let mut account: Account;
    loop {
        match sync_account(&ledger_db, &wallet_db, &account_id.to_string(), &logger) {
            Ok(_) => {}
            Err(SyncError::Database(WalletDbError::Diesel(
                diesel::result::Error::DatabaseError(kind, info),
            ))) => {
                match info.message() {
                    "database is locked" => log::trace!(logger, "Database locked. Will retry"),
                    _ => {
                        log::error!(
                            logger,
                            "Unexpected database error {:?} {:?} {:?} {:?} {:?} {:?}",
                            kind,
                            info,
                            info.details(),
                            info.column_name(),
                            info.table_name(),
                            info.hint(),
                        );
                        panic!("Could not manually sync account.");
                    }
                };
                std::thread::sleep(Duration::from_millis(500));
            }
            Err(e) => panic!("Could not sync account due to {:?}", e),
        }
        account = Account::get(&account_id, &wallet_db.get_conn().unwrap()).unwrap();
        if account.next_block_index as u64 == ledger_db.num_blocks().unwrap() {
            break;
        }
    }
    assert_eq!(account.next_block_index as u64, target_block_index);
    account
}

pub fn wait_for_sync(
    ledger_db: &LedgerDB,
    wallet_db: &WalletDb,
    account_id: &AccountID,
    target_block_index: u64,
) {
    let mut account: Account;
    loop {
        account = Account::get(&account_id, &wallet_db.get_conn().unwrap()).unwrap();
        if account.next_block_index as u64 == ledger_db.num_blocks().unwrap() {
            break;
        }
    }
    assert_eq!(account.next_block_index as u64, target_block_index);
}

pub fn setup_grpc_peer_manager_and_network_state(
    logger: Logger,
) -> (
    ConnectionManager<ThickClient<HardcodedCredentialsProvider>>,
    Arc<RwLock<PollingNetworkState<ThickClient<HardcodedCredentialsProvider>>>>,
) {
    let peer1 = test_client_uri(1);
    let peer2 = test_client_uri(2);
    let peers = vec![peer1.clone(), peer2.clone()];

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
                HardcodedCredentialsProvider::from(client_uri),
                logger.clone(),
            )
            .expect("Could not create thick client.")
        })
        .collect();

    let peer_manager = ConnectionManager::new(connected_peers, logger.clone());
    let quorum_set = QuorumSet::new_with_node_ids(
        2,
        vec![peer1.responder_id().unwrap(), peer2.responder_id().unwrap()],
    );
    let network_state = Arc::new(RwLock::new(PollingNetworkState::new(
        quorum_set,
        peer_manager.clone(),
        logger.clone(),
    )));

    {
        let mut network_state = network_state.write().unwrap();
        network_state.poll();
    }
    (peer_manager, network_state)
}

pub fn create_test_txo_for_recipient(
    recipient_account_key: &AccountKey,
    recipient_subaddress_index: u64,
    value: u64,
    rng: &mut StdRng,
) -> (TxOut, KeyImage) {
    let recipient = recipient_account_key.subaddress(recipient_subaddress_index);
    let tx_private_key = RistrettoPrivate::from_random(rng);
    let hint = EncryptedFogHint::fake_onetime_hint(rng);
    let tx_out = TxOut::new(value, &recipient, &tx_private_key, hint).unwrap();

    // Calculate KeyImage - note you cannot use KeyImage::from(tx_private_key)
    // because the calculation must be done with CryptoNote math (see
    // create_onetime_public_key and recover_onetime_private_key)
    let onetime_private_key = recover_onetime_private_key(
        &RistrettoPublic::try_from(&tx_out.public_key).unwrap(),
        recipient_account_key.view_private_key(),
        &recipient_account_key.subaddress_spend_private(recipient_subaddress_index),
    );
    assert_eq!(
        create_tx_out_target_key(&tx_private_key, &recipient),
        RistrettoPublic::from(&onetime_private_key)
    );

    let key_image = KeyImage::from(&onetime_private_key);
    (tx_out, key_image)
}

pub fn create_test_received_txo(
    account_key: &AccountKey,
    recipient_subaddress_index: u64,
    value: u64,
    received_block_index: u64,
    rng: &mut StdRng,
    wallet_db: &WalletDb,
) -> (String, TxOut, KeyImage) {
    let (txo, key_image) =
        create_test_txo_for_recipient(account_key, recipient_subaddress_index, value, rng);

    let txo_id_hex = Txo::create_received(
        txo.clone(),
        Some(recipient_subaddress_index as i64),
        Some(key_image),
        value,
        received_block_index as i64,
        &AccountID::from(account_key).to_string(),
        &wallet_db.get_conn().unwrap(),
    )
    .unwrap();
    (txo_id_hex, txo, key_image)
}

/// Creates a test minted and change txo.
///
/// Returns ((output_txo_id, value), (change_txo_id, value))
pub fn create_test_minted_and_change_txos(
    src_account_key: AccountKey,
    recipient: PublicAddress,
    value: u64,
    wallet_db: WalletDb,
    ledger_db: LedgerDB,
    logger: Logger,
) -> ((String, i64), (String, i64)) {
    let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

    // Use the builder to create valid TxOuts for this account
    let mut builder = WalletTransactionBuilder::<MockFogPubkeyResolver>::new(
        AccountID::from(&src_account_key).to_string(),
        wallet_db.clone(),
        ledger_db,
        get_resolver_factory(&mut rng).unwrap(),
        logger,
    );

    builder.add_recipient(recipient, value).unwrap();
    builder.select_txos(None).unwrap();
    builder.set_tombstone(0).unwrap();
    let tx_proposal = builder.build().unwrap();

    // There should be 2 outputs, one to dest and one change
    assert_eq!(tx_proposal.tx.prefix.outputs.len(), 2);

    // Create minted for the destination output.
    assert_eq!(tx_proposal.outlay_index_to_tx_out_index.len(), 1);
    let outlay_txo_index = tx_proposal.outlay_index_to_tx_out_index[&0];
    let tx_out = tx_proposal.tx.prefix.outputs[outlay_txo_index].clone();
    let processed_output = Txo::create_minted(
        &AccountID::from(&src_account_key).to_string(),
        &tx_out,
        &tx_proposal,
        outlay_txo_index,
        &wallet_db.get_conn().unwrap(),
    )
    .unwrap();
    assert!(processed_output.recipient.is_some());
    assert_eq!(processed_output.txo_type, TXO_USED_AS_OUTPUT);

    // Create minted for the change output.
    let change_txo_index = if outlay_txo_index == 0 { 1 } else { 0 };
    let change_tx_out = tx_proposal.tx.prefix.outputs[change_txo_index].clone();
    let processed_change = Txo::create_minted(
        &AccountID::from(&src_account_key).to_string(),
        &change_tx_out,
        &tx_proposal,
        change_txo_index,
        &wallet_db.get_conn().unwrap(),
    )
    .unwrap();
    assert_eq!(processed_change.recipient, None,);
    // Change starts as an output, and is updated to change when scanned.
    assert_eq!(processed_change.txo_type, TXO_USED_AS_CHANGE);
    (
        (processed_output.txo_id_hex, processed_output.value),
        (processed_change.txo_id_hex, processed_change.value),
    )
}

// Seed a local account with some Txos in the ledger
pub fn random_account_with_seed_values(
    wallet_db: &WalletDb,
    mut ledger_db: &mut LedgerDB,
    seed_values: &[u64],
    mut rng: &mut StdRng,
) -> AccountKey {
    let root_id = RootIdentity::from_random(&mut rng);
    let account_key = AccountKey::from(&root_id);
    {
        Account::create_from_root_entropy(
            &root_id.root_entropy,
            Some(0),
            None,
            None,
            &format!("SeedAccount{}", rng.next_u32()),
            None,
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
    }

    for value in seed_values.iter() {
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![account_key.subaddress(0)],
            *value,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );
    }

    wait_for_sync(
        &ledger_db,
        &wallet_db,
        &AccountID::from(&account_key),
        ledger_db.num_blocks().unwrap(),
    );

    // Make sure we have all our TXOs
    {
        assert_eq!(
            Txo::list_for_account(
                &AccountID::from(&account_key).to_string(),
                None,
                None,
                &wallet_db.get_conn().unwrap(),
            )
            .unwrap()
            .len(),
            seed_values.len(),
        );
    }

    account_key
}

pub fn builder_for_random_recipient(
    account_key: &AccountKey,
    wallet_db: &WalletDb,
    ledger_db: &LedgerDB,
    mut rng: &mut StdRng,
    logger: &Logger,
) -> (
    PublicAddress,
    WalletTransactionBuilder<MockFogPubkeyResolver>,
) {
    // Construct a transaction
    let builder: WalletTransactionBuilder<MockFogPubkeyResolver> = WalletTransactionBuilder::new(
        AccountID::from(account_key).to_string(),
        wallet_db.clone(),
        ledger_db.clone(),
        get_resolver_factory(&mut rng).unwrap(),
        logger.clone(),
    );

    let recipient_account_key = AccountKey::random(&mut rng);
    let recipient = recipient_account_key.subaddress(rng.next_u64());

    (recipient, builder)
}

pub fn get_resolver_factory(
    mut rng: &mut StdRng,
) -> Result<Arc<dyn Fn(&[FogUri]) -> Result<MockFogPubkeyResolver, String> + Send + Sync>, ()> {
    let fog_private_key = RistrettoPrivate::from_random(&mut rng);
    let fog_pubkey_resolver_factory: Arc<
        dyn Fn(&[FogUri]) -> Result<MockFogPubkeyResolver, String> + Send + Sync,
    > = Arc::new(move |_| -> Result<MockFogPubkeyResolver, String> {
        let mut fog_pubkey_resolver = MockFogPubkeyResolver::new();
        let pubkey = RistrettoPublic::from(&fog_private_key);
        fog_pubkey_resolver
            .expect_get_fog_pubkey()
            .return_once(move |_recipient| {
                Ok(FullyValidatedFogPubkey {
                    pubkey,
                    pubkey_expiry: 10000,
                })
            });
        Ok(fog_pubkey_resolver)
    });
    Ok(fog_pubkey_resolver_factory)
}

pub fn setup_wallet_service(
    ledger_db: LedgerDB,
    logger: Logger,
) -> WalletService<MockBlockchainConnection<LedgerDB>, MockFogPubkeyResolver> {
    let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

    let db_test_context = WalletDbTestContext::default();
    let wallet_db = db_test_context.get_db_instance(logger.clone());
    let (peer_manager, network_state) =
        setup_peer_manager_and_network_state(ledger_db.clone(), logger.clone());

    WalletService::new(
        wallet_db,
        ledger_db,
        peer_manager,
        network_state,
        get_resolver_factory(&mut rng).unwrap(),
        None,
        false,
        logger,
    )
}
