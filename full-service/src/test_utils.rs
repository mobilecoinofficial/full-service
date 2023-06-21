// Copyright (c) 2020-2021 MobileCoin Inc.
#[cfg(test)]
use crate::{
    config::NetworkConfig,
    db::{
        account::{AccountID, AccountModel},
        models::{Account, TransactionLog, Txo},
        transaction_log::TransactionLogModel,
        txo::TxoModel,
        WalletDb, WalletDbError,
    },
    error::SyncError,
    service::{
        models::tx_proposal::{TxProposal, UnsignedTxProposal},
        sync::sync_account_next_chunk,
        transaction::TransactionMemo,
        transaction_builder::WalletTransactionBuilder,
    },
    WalletService,
};
use diesel::{Connection as DSLConnection, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use mc_account_keys::{AccountKey, PublicAddress, RootIdentity};
use mc_blockchain_test_utils::make_block_metadata;
use mc_blockchain_types::{Block, BlockContents, BlockVersion};
use mc_common::logger::{log, Logger};
use mc_connection::{Connection, ConnectionManager};
use mc_connection_test_utils::{test_client_uri, MockBlockchainConnection};
use mc_consensus_scp::QuorumSet;
use mc_crypto_keys::{RistrettoPrivate, RistrettoPublic};
use mc_fog_report_validation::{FullyValidatedFogPubkey, MockFogPubkeyResolver};
use mc_ledger_db::{Ledger, LedgerDB};
use mc_ledger_sync::PollingNetworkState;
use mc_rand::{CryptoRng, RngCore};
use mc_transaction_core::{
    encrypted_fog_hint::EncryptedFogHint,
    onetime_keys::{create_tx_out_target_key, recover_onetime_private_key},
    ring_signature::KeyImage,
    tokens::Mob,
    tx::{Tx, TxOut},
    Amount, FeeMap, Token, TokenId,
};
use mc_util_from_random::FromRandom;
use mc_util_uri::{ConnectionUri, FogUri};
use rand::{distributions::Alphanumeric, rngs::StdRng, thread_rng, Rng, SeedableRng};
use std::{
    collections::BTreeMap,
    convert::TryFrom,
    env,
    ops::DerefMut,
    path::PathBuf,
    sync::{Arc, RwLock},
    time::Duration,
};
use tempdir::TempDir;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

pub const MOB: u64 = 1_000_000_000_000;

/// The amount each recipient gets in the test ledger.
pub const DEFAULT_PER_RECIPIENT_AMOUNT: u64 = 5_000 * MOB;

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
        env::set_var("MC_PASSWORD", "");
        env::set_var("MC_CHANGE_PASSWORD", "");
        let conn = &mut SqliteConnection::establish(&format!("{base_url}/{db_name}"))
            .unwrap_or_else(|err| panic!("Cannot connect to {} database: {:?}", db_name, err));
        // embedded_migrations::run(conn).expect("failed running migrations");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("failed running migrations");

        // Success
        Self { base_url, db_name }
    }
}

impl WalletDbTestContext {
    pub fn get_db_instance(&self, _logger: Logger) -> WalletDb {
        // Note: Setting db_connections too high results in IO Error: Too many open
        // files.
        WalletDb::new_from_url(&format!("{}/{}", self.base_url, self.db_name), 7)
            .expect("failed creating new SqlRecoveryDb")
    }
}

pub fn generate_n_blocks_on_ledger(
    num_random_recipients: u32,
    known_recipients: &[PublicAddress],
    num_blocks: usize,
    mut rng: &mut (impl CryptoRng + RngCore),
    mut ledger_db: &mut LedgerDB,
) {
    let mut public_addresses: Vec<PublicAddress> = (0..num_random_recipients)
        .map(|_i| mc_account_keys::AccountKey::random(&mut rng).default_subaddress())
        .collect();

    public_addresses.extend(known_recipients.iter().cloned());

    for _block_index in 0..num_blocks {
        let key_images = if ledger_db.num_blocks().unwrap() == 0 {
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
    let mut ledger_db = get_empty_test_ledger();
    generate_n_blocks_on_ledger(
        num_random_recipients,
        known_recipients,
        num_blocks,
        &mut rng,
        &mut ledger_db,
    );
    ledger_db
}

/// Set up an empty test ledger.
pub fn get_empty_test_ledger() -> LedgerDB {
    // Note that TempDir manages uniqueness by constructing paths
    // like: /tmp/ledger_db.tvF0XHTKsilx
    let ledger_db_tmp = TempDir::new("ledger_db").expect("Could not make tempdir for ledger db");
    let ledger_db_path = ledger_db_tmp
        .path()
        .to_str()
        .expect("Could not get path as string");
    generate_ledger_db(ledger_db_path)
}

/// Creates an empty LedgerDB.
///
/// # Arguments
/// * `path` - Path to the ledger's data.mdb file. If such a file exists, it
///   will be replaced.
pub fn generate_ledger_db(path: &str) -> LedgerDB {
    // DELETE the old database if it already exists.
    let _ = std::fs::remove_file(format!("{path}/data.mdb"));
    LedgerDB::create(&PathBuf::from(path)).expect("Could not create ledger_db");

    LedgerDB::open(&PathBuf::from(path)).expect("Could not open ledger_db")
}

fn append_test_block(
    ledger_db: &mut LedgerDB,
    block_contents: BlockContents,
    mut rng: &mut (impl CryptoRng + RngCore),
) -> u64 {
    let num_blocks = ledger_db.num_blocks().expect("failed to get block height");

    let new_block;
    if num_blocks > 0 {
        let parent = ledger_db
            .get_block(num_blocks - 1)
            .expect("failed to get parent block");
        new_block = Block::new_with_parent(
            BlockVersion::MAX,
            &parent,
            &Default::default(),
            &block_contents,
        );
    } else {
        new_block = Block::new_origin_block(&block_contents.outputs);
    }

    let block_metadata = make_block_metadata(new_block.id.clone(), &mut rng);

    ledger_db
        .append_block(&new_block, &block_contents, None, Some(&block_metadata))
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
                BlockVersion::MAX,
                Amount::new(output_value, Mob::ID),
                recipient,
                &RistrettoPrivate::from_random(rng),
                Default::default(),
            )
            .unwrap()
        })
        .collect();

    let block_contents = BlockContents {
        key_images: key_images.to_vec(),
        outputs,
        validated_mint_config_txs: Vec::new(),
        mint_txs: Vec::new(),
    };
    append_test_block(ledger_db, block_contents, rng)
}

pub fn add_block_with_tx(
    ledger_db: &mut LedgerDB,
    tx: Tx,
    rng: &mut (impl CryptoRng + RngCore),
) -> u64 {
    let block_contents = BlockContents {
        key_images: tx.key_images(),
        outputs: tx.prefix.outputs,
        validated_mint_config_txs: Vec::new(),
        mint_txs: Vec::new(),
    };
    append_test_block(ledger_db, block_contents, rng)
}

pub fn add_block_with_tx_outs(
    ledger_db: &mut LedgerDB,
    outputs: &[TxOut],
    key_images: &[KeyImage],
    rng: &mut (impl CryptoRng + RngCore),
) -> u64 {
    let block_contents = BlockContents {
        key_images: key_images.to_vec(),
        outputs: outputs.to_vec(),
        validated_mint_config_txs: Vec::new(),
        mint_txs: Vec::new(),
    };
    append_test_block(ledger_db, block_contents, rng)
}

pub fn setup_peer_manager_and_network_state(
    ledger_db: LedgerDB,
    logger: Logger,
    offline: bool,
) -> (
    ConnectionManager<MockBlockchainConnection<LedgerDB>>,
    Arc<RwLock<PollingNetworkState<MockBlockchainConnection<LedgerDB>>>>,
) {
    let (peers, node_ids) = if offline {
        (vec![], vec![])
    } else {
        let mut minimum_fees = BTreeMap::new();
        minimum_fees.insert(Mob::ID, Mob::MINIMUM_FEE);
        minimum_fees.insert(TokenId::from(1), 1024);
        let fee_map = FeeMap::try_from(minimum_fees).unwrap();

        let peer1 = MockBlockchainConnection::new(
            test_client_uri(1),
            ledger_db.clone(),
            0,
            fee_map.clone(),
        );
        let peer2 = MockBlockchainConnection::new(test_client_uri(2), ledger_db, 0, fee_map);

        (
            vec![peer1.clone(), peer2.clone()],
            vec![
                peer1.uri().host_and_port_responder_id().unwrap(),
                peer2.uri().host_and_port_responder_id().unwrap(),
            ],
        )
    };

    let peer_manager = ConnectionManager::new(peers, logger.clone());

    let quorum_set = QuorumSet::new_with_node_ids(2, node_ids);
    let network_state = Arc::new(RwLock::new(PollingNetworkState::new(
        quorum_set,
        peer_manager.clone(),
        logger,
    )));

    {
        let mut network_state = network_state.write().unwrap();
        network_state.poll();
    }

    (peer_manager, network_state)
}

// Sync account to most recent block
pub fn manually_sync_account(
    ledger_db: &LedgerDB,
    wallet_db: &WalletDb,
    account_id: &AccountID,
    logger: &Logger,
) -> Account {
    let mut account: Account;
    loop {
        match sync_account_next_chunk(
            ledger_db,
            &mut wallet_db.get_pooled_conn().unwrap().deref_mut(),
            &account_id.to_string(),
            logger,
        ) {
            Ok(_) => {}
            Err(SyncError::Database(WalletDbError::Diesel(
                diesel::result::Error::DatabaseError(_kind, info),
            ))) if info.message() == "database is locked" => {
                log::trace!(logger, "Database locked. Will retry");
                std::thread::sleep(Duration::from_millis(500));
            }
            Err(e) => panic!("Could not sync account due to {:?}", e),
        }
        account = Account::get(
            account_id,
            &mut wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();
        if account.next_block_index as u64 >= ledger_db.num_blocks().unwrap() {
            break;
        }
    }
    account
}

pub fn create_test_txo_for_recipient(
    recipient_account_key: &AccountKey,
    recipient_subaddress_index: u64,
    amount: Amount,
    rng: &mut StdRng,
) -> (TxOut, KeyImage) {
    let recipient = recipient_account_key.subaddress(recipient_subaddress_index);
    let tx_private_key = RistrettoPrivate::from_random(rng);
    let hint = EncryptedFogHint::fake_onetime_hint(rng);
    let tx_out = TxOut::new(BlockVersion::MAX, amount, &recipient, &tx_private_key, hint).unwrap();

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
    amount: Amount,
    received_block_index: u64,
    rng: &mut StdRng,
    wallet_db: &WalletDb,
) -> (String, TxOut, KeyImage) {
    let (txo, key_image) =
        create_test_txo_for_recipient(account_key, recipient_subaddress_index, amount, rng);

    let txo_id_hex = Txo::create_received(
        txo.clone(),
        Some(recipient_subaddress_index),
        Some(key_image),
        amount,
        received_block_index,
        &AccountID::from(account_key).to_string(),
        &mut wallet_db.get_pooled_conn().unwrap().deref_mut(),
    )
    .unwrap();
    (txo_id_hex, txo, key_image)
}

/// Creates a test minted and change txo.
///
/// Returns (txproposal, ((output_txo_id, value), (change_txo_id, value)))
pub fn create_test_minted_and_change_txos(
    src_account_key: AccountKey,
    recipient: PublicAddress,
    value: u64,
    wallet_db: WalletDb,
    ledger_db: LedgerDB,
) -> (TransactionLog, TxProposal) {
    let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

    // Use the builder to create valid TxOuts for this account
    let mut builder = WalletTransactionBuilder::<MockFogPubkeyResolver>::new(
        AccountID::from(&src_account_key).to_string(),
        ledger_db.clone(),
        get_resolver_factory(&mut rng).unwrap(),
    );

    let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
    let conn = pooled_conn.deref_mut();
    builder.add_recipient(recipient, value, Mob::ID).unwrap();
    builder.select_txos(conn, None).unwrap();
    builder.set_tombstone(0).unwrap();
    let unsigned_tx_proposal = builder
        .build(TransactionMemo::RTH(None, None), conn)
        .unwrap();
    let tx_proposal = unsigned_tx_proposal.sign(&src_account_key, None).unwrap();

    // There should be 2 outputs, one to dest and one change
    assert_eq!(tx_proposal.tx.prefix.outputs.len(), 2);

    (
        TransactionLog::log_submitted(
            &tx_proposal,
            ledger_db.num_blocks().unwrap(),
            "".to_string(),
            &AccountID::from(&src_account_key).to_string(),
            conn,
        )
        .unwrap(),
        tx_proposal,
    )
}

/// Creates an unsigned transaction proposal
///
/// Returns (txproposal, ((output_txo_id, value), (change_txo_id, value)))
pub fn create_test_unsigned_txproposal_and_log(
    src_account_key: AccountKey,
    recipient: PublicAddress,
    value: u64,
    wallet_db: WalletDb,
    ledger_db: LedgerDB,
) -> (TransactionLog, UnsignedTxProposal) {
    let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

    // Use the builder to create valid TxOuts for this account
    let mut builder = WalletTransactionBuilder::<MockFogPubkeyResolver>::new(
        AccountID::from(&src_account_key).to_string(),
        ledger_db,
        get_resolver_factory(&mut rng).unwrap(),
    );

    let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
    let conn = pooled_conn.deref_mut();
    builder.add_recipient(recipient, value, Mob::ID).unwrap();
    builder.select_txos(conn, None).unwrap();
    builder.set_tombstone(0).unwrap();
    let unsigned_tx_proposal = builder
        .build(TransactionMemo::RTH(None, None), conn)
        .unwrap();

    (
        TransactionLog::log_built(
            &unsigned_tx_proposal,
            &AccountID::from(&src_account_key),
            conn,
        )
        .unwrap(),
        unsigned_tx_proposal,
    )
}

// Seed a local account with some Txos in the ledger
pub fn random_account_with_seed_values(
    wallet_db: &WalletDb,
    ledger_db: &mut LedgerDB,
    seed_values: &[u64],
    mut rng: &mut StdRng,
    logger: &Logger,
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
            "".to_string(),
            "".to_string(),
            &mut wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();
    }

    for value in seed_values.iter() {
        add_block_to_ledger_db(
            ledger_db,
            &vec![account_key.subaddress(0)],
            *value,
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );
    }

    manually_sync_account(ledger_db, wallet_db, &AccountID::from(&account_key), logger);

    // Make sure we have all our TXOs
    {
        assert_eq!(
            Txo::list_for_account(
                &AccountID::from(&account_key).to_string(),
                None,
                None,
                None,
                None,
                None,
                Some(0),
                &mut wallet_db.get_pooled_conn().unwrap().deref_mut(),
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
    ledger_db: &LedgerDB,
    mut rng: &mut StdRng,
) -> (
    PublicAddress,
    WalletTransactionBuilder<MockFogPubkeyResolver>,
) {
    // Construct a transaction
    let builder: WalletTransactionBuilder<MockFogPubkeyResolver> = WalletTransactionBuilder::new(
        AccountID::from(account_key).to_string(),
        ledger_db.clone(),
        get_resolver_factory(rng).unwrap(),
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
            .returning(move |_| {
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
    setup_wallet_service_impl(ledger_db, logger, false, false)
}

pub fn setup_wallet_service_offline(
    ledger_db: LedgerDB,
    logger: Logger,
) -> WalletService<MockBlockchainConnection<LedgerDB>, MockFogPubkeyResolver> {
    setup_wallet_service_impl(ledger_db, logger, true, false)
}

fn setup_wallet_service_impl(
    ledger_db: LedgerDB,
    logger: Logger,
    offline: bool,
    no_wallet_db: bool,
) -> WalletService<MockBlockchainConnection<LedgerDB>, MockFogPubkeyResolver> {
    let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

    let db_test_context = WalletDbTestContext::default();
    let wallet_db = match no_wallet_db {
        true => None,
        false => Some(db_test_context.get_db_instance(logger.clone())),
    };
    let (peer_manager, network_state) =
        setup_peer_manager_and_network_state(ledger_db.clone(), logger.clone(), offline);

    let network_setup_config = NetworkConfig {
        offline,
        chain_id: "rust_tests".to_string(),
        peers: None,
        tx_sources: None,
    };

    WalletService::new(
        wallet_db,
        ledger_db,
        None,
        peer_manager,
        network_setup_config,
        network_state,
        get_resolver_factory(&mut rng).unwrap(),
        offline,
        logger,
    )
}
