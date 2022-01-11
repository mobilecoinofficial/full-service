// Copyright (c) 2020-2021 MobileCoin Inc.

//! A builder for transactions from the wallet. Note that we have a
//! TransactionBuilder in the MobileCoin transaction crate, but that is a lower
//! level of building, once you have already obtained all of the materials that
//! go into a transaction.
//!
//! This module, on the other hand, builds a transaction within the context of
//! the wallet.

use crate::{
    db::{
        account::{AccountID, AccountModel},
        models::{Account, Txo},
        txo::TxoModel,
        WalletDb,
    },
    error::WalletTransactionBuilderError,
};
use mc_account_keys::{AccountKey, PublicAddress};
use mc_common::{
    logger::{log, Logger},
    HashMap, HashSet,
};
use mc_crypto_keys::RistrettoPublic;
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::{Ledger, LedgerDB};
use mc_mobilecoind::{
    payments::{Outlay, TxProposal},
    UnspentTxOut,
};
use mc_transaction_core::{
    constants::{MINIMUM_FEE, RING_SIZE},
    onetime_keys::recover_onetime_private_key,
    ring_signature::KeyImage,
    tx::{TxOut, TxOutMembershipProof},
};
use mc_transaction_std::{InputCredentials, NoMemoBuilder, TransactionBuilder};
use mc_util_uri::FogUri;

use diesel::prelude::*;
use rand::Rng;
use std::{convert::TryFrom, str::FromStr, sync::Arc};

/// Default number of blocks used for calculating transaction tombstone block
/// number.
// TODO support for making this configurable
pub const DEFAULT_NEW_TX_BLOCK_ATTEMPTS: u64 = 50;

/// A builder of transactions constructed from this wallet.
pub struct WalletTransactionBuilder<FPR: FogPubkeyResolver + 'static> {
    /// Account ID (hex-encoded) from which to construct a transaction.
    account_id_hex: String,

    /// The wallet DB.
    wallet_db: WalletDb,

    /// The ledger DB.
    ledger_db: LedgerDB,

    /// Optional inputs specified to use to construct the transaction.
    inputs: Vec<Txo>,

    /// Vector of (PublicAddress, Amounts) for the recipients of this
    /// transaction.
    outlays: Vec<(PublicAddress, u64)>,

    /// The block after which this transaction is invalid.
    tombstone: u64,

    /// The fee for the transaction.
    fee: Option<u64>,

    /// Fog resolver maker, used when constructing outputs to fog recipients.
    /// This is abstracted because in tests, we don't want to form grpc
    /// connections to fog.
    fog_resolver_factory: Arc<dyn Fn(&[FogUri]) -> Result<FPR, String> + Send + Sync>,

    /// Logger.
    logger: Logger,
}

impl<FPR: FogPubkeyResolver + 'static> WalletTransactionBuilder<FPR> {
    pub fn new(
        account_id_hex: String,
        wallet_db: WalletDb,
        ledger_db: LedgerDB,
        fog_resolver_factory: Arc<dyn Fn(&[FogUri]) -> Result<FPR, String> + Send + Sync + 'static>,
        logger: Logger,
    ) -> Self {
        WalletTransactionBuilder {
            account_id_hex,
            wallet_db,
            ledger_db,
            inputs: vec![],
            outlays: vec![],
            tombstone: 0,
            fee: None,
            fog_resolver_factory,
            logger,
        }
    }

    /// Sets inputs to the txos associated with the given txo_ids. Only unspent
    /// txos are included.
    pub fn set_txos(
        &mut self,
        input_txo_ids: &[String],
    ) -> Result<(), WalletTransactionBuilderError> {
        let txos = Txo::select_by_id(&input_txo_ids.to_vec(), &self.wallet_db.get_conn()?)?;
        let unspent: Vec<Txo> = txos
            .iter()
            .filter(|txo| {
                txo.pending_tombstone_block_index == None && txo.spent_block_index == None
            })
            .cloned()
            .collect();
        if unspent.iter().map(|t| t.value as u128).sum::<u128>() > u64::MAX as u128 {
            return Err(WalletTransactionBuilderError::OutboundValueTooLarge);
        }
        self.inputs = unspent;
        Ok(())
    }

    /// Selects Txos from the account.
    pub fn select_txos(
        &mut self,
        max_spendable_value: Option<u64>,
    ) -> Result<(), WalletTransactionBuilderError> {
        let outlay_value_sum = self.outlays.iter().map(|(_r, v)| *v as u128).sum::<u128>();

        let fee = self.fee.unwrap_or(MINIMUM_FEE);
        if outlay_value_sum > u64::MAX as u128 || outlay_value_sum > u64::MAX as u128 - fee as u128
        {
            return Err(WalletTransactionBuilderError::OutboundValueTooLarge);
        }
        log::info!(
            self.logger,
            "Selecting Txos for value {:?} with fee {:?}",
            outlay_value_sum,
            fee
        );
        let total_value = outlay_value_sum as u64 + fee;
        self.inputs = Txo::select_unspent_txos_for_value(
            &self.account_id_hex,
            total_value,
            max_spendable_value.map(|v| v as i64),
            &self.wallet_db.get_conn()?,
        )?;

        Ok(())
    }

    pub fn add_recipient(
        &mut self,
        recipient: PublicAddress,
        value: u64,
    ) -> Result<(), WalletTransactionBuilderError> {
        // Verify that the maximum output value of this transaction remains under
        // u64::MAX
        let cur_sum = self.outlays.iter().map(|(_r, v)| *v as u128).sum::<u128>();
        if cur_sum > u64::MAX as u128 {
            return Err(WalletTransactionBuilderError::OutboundValueTooLarge);
        }
        self.outlays.push((recipient, value));
        Ok(())
    }

    pub fn set_fee(&mut self, fee: u64) -> Result<(), WalletTransactionBuilderError> {
        if fee < 1 {
            return Err(WalletTransactionBuilderError::InsufficientFee(
                "1".to_string(),
            ));
        }
        self.fee = Some(fee);
        Ok(())
    }

    pub fn set_tombstone(&mut self, tombstone: u64) -> Result<(), WalletTransactionBuilderError> {
        let tombstone_block = if tombstone > 0 {
            tombstone
        } else {
            let num_blocks_in_ledger = self.ledger_db.num_blocks()?;
            num_blocks_in_ledger + DEFAULT_NEW_TX_BLOCK_ATTEMPTS
        };
        self.tombstone = tombstone_block;
        Ok(())
    }

    /// Consumes self
    pub fn build(&self) -> Result<TxProposal, WalletTransactionBuilderError> {
        if self.inputs.is_empty() {
            return Err(WalletTransactionBuilderError::NoInputs);
        }

        if self.tombstone == 0 {
            return Err(WalletTransactionBuilderError::TombstoneNotSet);
        }

        let conn = self.wallet_db.get_conn()?;

        conn.transaction::<TxProposal, WalletTransactionBuilderError, _>(|| {
            let account: Account =
                Account::get(&AccountID(self.account_id_hex.to_string()), &conn)?;
            let from_account_key: AccountKey = mc_util_serial::decode(&account.account_key)?;

            // Collect all required FogUris from public addresses, then pass to resolver
            // factory
            let fog_resolver = {
                let change_address =
                    from_account_key.subaddress(account.change_subaddress_index as u64);
                let fog_uris = core::slice::from_ref(&change_address)
                    .iter()
                    .chain(self.outlays.iter().map(|(receiver, _amount)| receiver))
                    .filter_map(|x| extract_fog_uri(x).transpose())
                    .collect::<Result<Vec<_>, _>>()?;
                (self.fog_resolver_factory)(&fog_uris)
                    .map_err(WalletTransactionBuilderError::FogPubkeyResolver)?
            };

            // Create transaction builder.
            // TODO: After servers that support memos are deployed, use RTHMemoBuilder here
            let memo_builder = NoMemoBuilder::default();
            let mut transaction_builder = TransactionBuilder::new(fog_resolver, memo_builder);
            transaction_builder.set_fee(self.fee.unwrap_or(MINIMUM_FEE))?;

            // Get membership proofs for our inputs
            let indexes = self
                .inputs
                .iter()
                .map(|utxo| {
                    let txo: TxOut = mc_util_serial::decode(&utxo.txo)?;
                    self.ledger_db.get_tx_out_index_by_hash(&txo.hash())
                })
                .collect::<Result<Vec<u64>, mc_ledger_db::Error>>()?;
            let proofs = self.ledger_db.get_tx_out_proof_of_memberships(&indexes)?;

            let inputs_and_proofs: Vec<(Txo, TxOutMembershipProof)> = self
                .inputs
                .clone()
                .into_iter()
                .zip(proofs.into_iter())
                .collect();

            let excluded_tx_out_indices: Vec<u64> = inputs_and_proofs
                .iter()
                .map(|(utxo, _membership_proof)| {
                    let txo: TxOut = mc_util_serial::decode(&utxo.txo)?;
                    self.ledger_db
                        .get_tx_out_index_by_hash(&txo.hash())
                        .map_err(WalletTransactionBuilderError::LedgerDB)
                })
                .collect::<Result<Vec<u64>, WalletTransactionBuilderError>>()?;

            let rings = self.get_rings(inputs_and_proofs.len(), &excluded_tx_out_indices)?;

            if rings.len() != inputs_and_proofs.len() {
                return Err(WalletTransactionBuilderError::RingSizeMismatch);
            }

            if self.outlays.is_empty() {
                return Err(WalletTransactionBuilderError::NoRecipient);
            }

            // Unzip each vec of tuples into a tuple of vecs.
            let mut rings_and_proofs: Vec<(Vec<TxOut>, Vec<TxOutMembershipProof>)> = rings
                .into_iter()
                .map(|tuples| tuples.into_iter().unzip())
                .collect();

            // Add inputs to the tx.
            for (utxo, proof) in inputs_and_proofs.iter() {
                let db_tx_out: TxOut = mc_util_serial::decode(&utxo.txo)?;
                let (mut ring, mut membership_proofs) = rings_and_proofs
                    .pop()
                    .ok_or(WalletTransactionBuilderError::RingsAndProofsEmpty)?;
                if ring.len() != membership_proofs.len() {
                    return Err(WalletTransactionBuilderError::RingSizeMismatch);
                }

                // Add the input to the ring.
                let position_opt = ring.iter().position(|txo| *txo == db_tx_out);
                let real_key_index = match position_opt {
                    Some(position) => {
                        // The input is already present in the ring.
                        // This could happen if ring elements are sampled randomly from the
                        // ledger.
                        position
                    }
                    None => {
                        // The input is not already in the ring.
                        if ring.is_empty() {
                            // Append the input and its proof of membership.
                            ring.push(db_tx_out.clone());
                            membership_proofs.push(proof.clone());
                        } else {
                            // Replace the first element of the ring.
                            ring[0] = db_tx_out.clone();
                            membership_proofs[0] = proof.clone();
                        }
                        // The real input is always the first element. This is safe because
                        // TransactionBuilder sorts each ring.
                        0
                    }
                };

                if ring.len() != membership_proofs.len() {
                    return Err(WalletTransactionBuilderError::RingSizeMismatch);
                }

                let public_key = RistrettoPublic::try_from(&db_tx_out.public_key).unwrap();

                let subaddress_index = if let Some(s) = utxo.subaddress_index {
                    s
                } else {
                    return Err(WalletTransactionBuilderError::NullSubaddress(
                        utxo.txo_id_hex.to_string(),
                    ));
                };

                let onetime_private_key = recover_onetime_private_key(
                    &public_key,
                    from_account_key.view_private_key(),
                    &from_account_key.subaddress_spend_private(subaddress_index as u64),
                );

                let key_image = KeyImage::from(&onetime_private_key);
                log::debug!(
                    self.logger,
                    "Adding input: ring {:?}, utxo index {:?}, key image {:?}, pubkey {:?}",
                    ring,
                    real_key_index,
                    key_image,
                    public_key
                );

                transaction_builder.add_input(InputCredentials::new(
                    ring,
                    membership_proofs,
                    real_key_index,
                    onetime_private_key,
                    *from_account_key.view_private_key(),
                )?);
            }

            // Add outputs to our destinations.
            // Note that we make an assumption currently when logging submitted Txos that
            // they were built  with only one recipient, and one change txo.
            let mut total_value = 0;
            let mut tx_out_to_outlay_index: HashMap<TxOut, usize> = HashMap::default();
            let mut outlay_confirmation_numbers = Vec::default();
            let mut rng = rand::thread_rng();
            for (i, (recipient, out_value)) in self.outlays.iter().enumerate() {
                let (tx_out, confirmation_number) =
                    transaction_builder.add_output(*out_value as u64, recipient, &mut rng)?;

                tx_out_to_outlay_index.insert(tx_out, i);
                outlay_confirmation_numbers.push(confirmation_number);

                total_value += *out_value;
            }

            // Figure out if we have change.
            let input_value = inputs_and_proofs
                .iter()
                .fold(0, |acc, (utxo, _proof)| acc + utxo.value);
            if (total_value + transaction_builder.get_fee()) > input_value as u64 {
                return Err(WalletTransactionBuilderError::InsufficientInputFunds(
                    format!(
                        "Total value required to send transaction {:?}, but only {:?} in inputs",
                        total_value + transaction_builder.get_fee(),
                        input_value
                    ),
                ));
            }

            let change = input_value as u64 - total_value - transaction_builder.get_fee();

            // If we do, add an output for that as well.
            if change > 0 {
                let change_public_address =
                    from_account_key.subaddress(account.change_subaddress_index as u64);
                // FIXME: verify that fog resolver knows to send change with hint encrypted to
                // the main public address
                transaction_builder.add_output(change, &change_public_address, &mut rng)?;
                // FIXME: CBB - map error to indicate error with change
            }

            // Set tombstone block.
            transaction_builder.set_tombstone_block(self.tombstone);

            // Build tx.
            let tx = transaction_builder.build(&mut rng)?;

            // Map each TxOut in the constructed transaction to its respective outlay.
            let outlay_index_to_tx_out_index: HashMap<usize, usize> = tx
                .prefix
                .outputs
                .iter()
                .enumerate()
                .filter_map(|(tx_out_index, tx_out)| {
                    tx_out_to_outlay_index
                        .get(tx_out)
                        .map(|outlay_index| (*outlay_index, tx_out_index))
                })
                .collect();

            // Sanity check: All of our outlays should have a unique index in the map.
            assert_eq!(outlay_index_to_tx_out_index.len(), self.outlays.len());
            let mut found_tx_out_indices: HashSet<&usize> = HashSet::default();
            for i in 0..self.outlays.len() {
                let tx_out_index = outlay_index_to_tx_out_index
                    .get(&i)
                    .expect("index not in map");
                if !found_tx_out_indices.insert(tx_out_index) {
                    panic!("duplicate index {} found in map", tx_out_index);
                }
            }

            // Make the UnspentTxOut for each Txo
            // FIXME: WS-27 - I would prefer to provide just the txo_id_hex per txout, but
            // this at least preserves some interoperability between
            // mobilecoind and wallet-service. However, this is
            // pretty clunky and I would rather not expose a storage
            // type from mobilecoind just to get around having to write a bunch of
            // tedious json conversions.
            // Return the TxProposal
            let selected_utxos = inputs_and_proofs
                    .iter()
                    .map(|(utxo, _membership_proof)| {
                        let decoded_tx_out = mc_util_serial::decode(&utxo.txo).unwrap();
                        let decoded_key_image =
                            mc_util_serial::decode(&utxo.key_image.clone().unwrap()).unwrap();

                        UnspentTxOut {
                            tx_out: decoded_tx_out,
                            subaddress_index: utxo.subaddress_index.unwrap() as u64, // verified not null earlier
                            key_image: decoded_key_image,
                            value: utxo.value as u64,
                            attempted_spend_height: 0, // NOTE: these are null because not tracked here
                            attempted_spend_tombstone: 0,
                        }
                    })
                    .collect();
            Ok(TxProposal {
                utxos: selected_utxos,
                outlays: self
                    .outlays
                    .iter()
                    .map(|(recipient, value)| Outlay {
                        receiver: recipient.clone(),
                        value: *value,
                    })
                    .collect::<Vec<Outlay>>(),
                tx,
                outlay_index_to_tx_out_index,
                outlay_confirmation_numbers,
            })
        })
    }

    /// Get rings.
    fn get_rings(
        &self,
        num_rings: usize,
        excluded_tx_out_indices: &[u64],
    ) -> Result<Vec<Vec<(TxOut, TxOutMembershipProof)>>, WalletTransactionBuilderError> {
        let num_requested = RING_SIZE * num_rings;
        let num_txos = self.ledger_db.num_txos()?;

        // Check that the ledger contains enough tx outs.
        if excluded_tx_out_indices.len() as u64 > num_txos {
            return Err(WalletTransactionBuilderError::InvalidArgument(
                "excluded_tx_out_indices exceeds amount of tx outs in ledger".to_string(),
            ));
        }

        if num_requested > (num_txos as usize - excluded_tx_out_indices.len()) {
            return Err(WalletTransactionBuilderError::InsufficientTxOuts);
        }

        // Randomly sample `num_requested` TxOuts, without replacement and convert into
        // a Vec<u64>
        let mut rng = rand::thread_rng();
        let mut sampled_indices: HashSet<u64> = HashSet::default();
        while sampled_indices.len() < num_requested {
            let index = rng.gen_range(0..num_txos);
            if excluded_tx_out_indices.contains(&index) {
                continue;
            }
            sampled_indices.insert(index);
        }
        let sampled_indices_vec: Vec<u64> = sampled_indices.into_iter().collect();

        // Get proofs for all of those indexes.
        let proofs = self
            .ledger_db
            .get_tx_out_proof_of_memberships(&sampled_indices_vec)?;

        // Create an iterator that returns (index, proof) elements.
        let mut indexes_and_proofs_iterator =
            sampled_indices_vec.into_iter().zip(proofs.into_iter());

        // Convert that into a Vec<Vec<TxOut, TxOutMembershipProof>>
        let mut rings_with_proofs = Vec::new();

        for _ in 0..num_rings {
            let mut ring = Vec::new();
            for _ in 0..RING_SIZE {
                let (index, proof) = indexes_and_proofs_iterator.next().unwrap();
                let tx_out = self.ledger_db.get_tx_out_by_index(index)?;

                ring.push((tx_out, proof));
            }
            rings_with_proofs.push(ring);
        }

        Ok(rings_with_proofs)
    }
}

// Helper which extracts FogUri from PublicAddress or returns None, or returns
// an error
fn extract_fog_uri(addr: &PublicAddress) -> Result<Option<FogUri>, WalletTransactionBuilderError> {
    if let Some(string) = addr.fog_report_url() {
        Ok(Some(FogUri::from_str(string)?))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::WalletDbError,
        service::sync::SyncThread,
        test_utils::{
            builder_for_random_recipient, get_test_ledger, random_account_with_seed_values,
            WalletDbTestContext, MOB,
        },
    };
    use mc_common::logger::{test_with_logger, Logger};
    use rand::{rngs::StdRng, SeedableRng};

    #[test_with_logger]
    fn test_build_with_utxos(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread =
            SyncThread::start(ledger_db.clone(), wallet_db.clone(), None, logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![
                11 * MOB as u64,
                11 * MOB as u64,
                11 * MOB as u64,
                111111 * MOB as u64,
            ],
            &mut rng,
        );

        // Construct a transaction
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &wallet_db, &ledger_db, &mut rng, &logger);

        // Send value specifically for your smallest Txo size. Should take 2 inputs
        // and also make change.
        let value = 11 * MOB as u64;
        builder.add_recipient(recipient.clone(), value).unwrap();

        // Select the txos for the recipient
        builder.select_txos(None).unwrap();
        builder.set_tombstone(0).unwrap();

        let proposal = builder.build().unwrap();
        assert_eq!(proposal.outlays.len(), 1);
        assert_eq!(proposal.outlays[0].receiver, recipient);
        assert_eq!(proposal.outlays[0].value, value);
        assert_eq!(proposal.tx.prefix.inputs.len(), 2);
        assert_eq!(proposal.tx.prefix.fee, MINIMUM_FEE);
        assert_eq!(proposal.tx.prefix.outputs.len(), 2);
    }

    // Test that large values are handled correctly.
    #[test_with_logger]
    fn test_big_values(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread =
            SyncThread::start(ledger_db.clone(), wallet_db.clone(), None, logger.clone());

        // Give ourselves enough MOB that we have more than u64::MAX, 18_446_745 MOB
        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![
                7_000_000 * MOB as u64,
                7_000_000 * MOB as u64,
                7_000_000 * MOB as u64,
            ],
            &mut rng,
        );

        // Check balance
        let unspent = Txo::list_unspent(
            &AccountID::from(&account_key).to_string(),
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        let balance: u128 = unspent.iter().map(|t| t.value as u128).sum::<u128>();
        assert_eq!(balance, 21_000_000 * MOB as u128);

        // Now try to send a transaction with a value > u64::MAX
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &wallet_db, &ledger_db, &mut rng, &logger);

        let value = u64::MAX;
        builder.add_recipient(recipient.clone(), value).unwrap();

        // Select the txos for the recipient - should error because > u64::MAX
        match builder.select_txos(None) {
            Ok(_) => panic!("Should not be allowed to construct outbound values > u64::MAX"),
            Err(WalletTransactionBuilderError::OutboundValueTooLarge) => {}
            Err(e) => panic!("Unexpected error {:?}", e),
        }
    }

    // Users should be able to set the txos specifically that they want to send
    #[test_with_logger]
    fn test_setting_txos(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread =
            SyncThread::start(ledger_db.clone(), wallet_db.clone(), None, logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![70 * MOB as u64, 80 * MOB as u64, 90 * MOB as u64],
            &mut rng,
        );

        // Get our TXO list
        let txos: Vec<Txo> = Txo::list_for_account(
            &AccountID::from(&account_key).to_string(),
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &wallet_db, &ledger_db, &mut rng, &logger);

        // Setting value to exactly the input will fail because you need funds for fee
        builder
            .add_recipient(recipient.clone(), txos[0].value as u64)
            .unwrap();

        builder.set_txos(&vec![txos[0].txo_id_hex.clone()]).unwrap();
        builder.set_tombstone(0).unwrap();
        match builder.build() {
            Ok(_) => {
                panic!("Should not be able to construct Tx with > inputs value as output value")
            }
            Err(WalletTransactionBuilderError::InsufficientInputFunds(_)) => {}
            Err(e) => panic!("Unexpected error {:?}", e),
        }

        // Now build, setting to multiple TXOs
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &wallet_db, &ledger_db, &mut rng, &logger);

        // Set value to just slightly more than what fits in the one TXO
        builder
            .add_recipient(recipient.clone(), txos[0].value as u64 + 10)
            .unwrap();

        builder
            .set_txos(&vec![
                txos[0].txo_id_hex.clone(),
                txos[1].txo_id_hex.clone(),
            ])
            .unwrap();
        builder.set_tombstone(0).unwrap();
        let proposal = builder.build().unwrap();
        assert_eq!(proposal.outlays.len(), 1);
        assert_eq!(proposal.outlays[0].receiver, recipient);
        assert_eq!(proposal.outlays[0].value, txos[0].value as u64 + 10);
        assert_eq!(proposal.tx.prefix.inputs.len(), 2); // need one more for fee
        assert_eq!(proposal.tx.prefix.fee, MINIMUM_FEE);
        assert_eq!(proposal.tx.prefix.outputs.len(), 2); // self and change
    }

    // Test max_spendable correctly filters out txos above max_spendable
    #[test_with_logger]
    fn test_max_spendable(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread =
            SyncThread::start(ledger_db.clone(), wallet_db.clone(), None, logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![70 * MOB as u64, 80 * MOB as u64, 90 * MOB as u64],
            &mut rng,
        );

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &wallet_db, &ledger_db, &mut rng, &logger);

        // Setting value to exactly the input will fail because you need funds for fee
        builder
            .add_recipient(recipient.clone(), 80 * MOB as u64)
            .unwrap();

        // Test that selecting Txos with max_spendable < all our txo values fails
        match builder.select_txos(Some(10)) {
            Ok(_) => panic!("Should not be able to construct tx when max_spendable < all txos"),
            Err(WalletTransactionBuilderError::WalletDb(WalletDbError::NoSpendableTxos)) => {}
            Err(e) => panic!("Unexpected error {:?}", e),
        }

        // We should be able to try again, with max_spendable at 70, but will not hit
        // our outlay target (80 * MOB)
        match builder.select_txos(Some(70 * MOB as u64)) {
            Ok(_) => panic!("Should not be able to construct tx when max_spendable < all txos"),
            Err(WalletTransactionBuilderError::WalletDb(
                WalletDbError::InsufficientFundsUnderMaxSpendable(_),
            )) => {}
            Err(e) => panic!("Unexpected error {:?}", e),
        }

        // Now, we should succeed if we set max_spendable = 80 * MOB, because we will
        // pick up both 70 and 80
        builder.select_txos(Some(80 * MOB as u64)).unwrap();
        builder.set_tombstone(0).unwrap();
        let proposal = builder.build().unwrap();
        assert_eq!(proposal.outlays.len(), 1);
        assert_eq!(proposal.outlays[0].receiver, recipient);
        assert_eq!(proposal.outlays[0].value, 80 * MOB as u64);
        assert_eq!(proposal.tx.prefix.inputs.len(), 2); // uses both 70 and 80
        assert_eq!(proposal.tx.prefix.fee, MINIMUM_FEE);
        assert_eq!(proposal.tx.prefix.outputs.len(), 2); // self and change
    }

    // Test setting and not setting tombstone block
    #[test_with_logger]
    fn test_tombstone(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread =
            SyncThread::start(ledger_db.clone(), wallet_db.clone(), None, logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![70 * MOB as u64],
            &mut rng,
        );

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &wallet_db, &ledger_db, &mut rng, &logger);

        builder
            .add_recipient(recipient.clone(), 10 * MOB as u64)
            .unwrap();
        builder.select_txos(None).unwrap();

        // Sanity check that our ledger is the height we think it is
        assert_eq!(ledger_db.num_blocks().unwrap(), 13);

        // We must set tombstone block before building
        match builder.build() {
            Ok(_) => panic!("Expected TombstoneNotSet error"),
            Err(WalletTransactionBuilderError::TombstoneNotSet) => {}
            Err(e) => panic!("Unexpected error {:?}", e),
        }

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &wallet_db, &ledger_db, &mut rng, &logger);

        builder
            .add_recipient(recipient.clone(), 10 * MOB as u64)
            .unwrap();
        builder.select_txos(None).unwrap();

        // Set to default
        builder.set_tombstone(0).unwrap();

        // Not setting the tombstone results in tombstone = 0. This is an acceptable
        // value,
        let proposal = builder.build().unwrap();
        assert_eq!(proposal.tx.prefix.tombstone_block, 63);

        // Build a transaction and explicitly set tombstone
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &wallet_db, &ledger_db, &mut rng, &logger);

        builder
            .add_recipient(recipient.clone(), 10 * MOB as u64)
            .unwrap();
        builder.select_txos(None).unwrap();

        // Set to default
        builder.set_tombstone(20).unwrap();

        // Not setting the tombstone results in tombstone = 0. This is an acceptable
        // value,
        let proposal = builder.build().unwrap();
        assert_eq!(proposal.tx.prefix.tombstone_block, 20);
    }

    // Test setting and not setting the fee
    #[test_with_logger]
    fn test_fee(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread =
            SyncThread::start(ledger_db.clone(), wallet_db.clone(), None, logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![70 * MOB as u64],
            &mut rng,
        );

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &wallet_db, &ledger_db, &mut rng, &logger);

        builder
            .add_recipient(recipient.clone(), 10 * MOB as u64)
            .unwrap();
        builder.select_txos(None).unwrap();
        builder.set_tombstone(0).unwrap();

        // Verify that not setting fee results in default fee
        let proposal = builder.build().unwrap();
        assert_eq!(proposal.tx.prefix.fee, MINIMUM_FEE);

        // You cannot set fee to 0
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &wallet_db, &ledger_db, &mut rng, &logger);

        builder
            .add_recipient(recipient.clone(), 10 * MOB as u64)
            .unwrap();
        builder.select_txos(None).unwrap();
        builder.set_tombstone(0).unwrap();
        match builder.set_fee(0) {
            Ok(_) => panic!("Should not be able to set fee to 0"),
            Err(WalletTransactionBuilderError::InsufficientFee(_)) => {}
            Err(e) => panic!("Unexpected error {:?}", e),
        }

        // Verify that not setting fee results in default fee
        let proposal = builder.build().unwrap();
        assert_eq!(proposal.tx.prefix.fee, MINIMUM_FEE);

        // Setting fee less than minimum fee should fail
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &wallet_db, &ledger_db, &mut rng, &logger);

        builder
            .add_recipient(recipient.clone(), 10 * MOB as u64)
            .unwrap();
        builder.select_txos(None).unwrap();
        builder.set_tombstone(0).unwrap();
        match builder.set_fee(0) {
            Ok(_) => panic!("Should not be able to set fee to 0"),
            Err(WalletTransactionBuilderError::InsufficientFee(_)) => {}
            Err(e) => panic!("Unexpected error {:?}", e),
        }

        // Setting fee greater than MINIMUM_FEE works
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &wallet_db, &ledger_db, &mut rng, &logger);

        builder
            .add_recipient(recipient.clone(), 10 * MOB as u64)
            .unwrap();
        builder.select_txos(None).unwrap();
        builder.set_tombstone(0).unwrap();
        builder.set_fee(MINIMUM_FEE * 10).unwrap();
        let proposal = builder.build().unwrap();
        assert_eq!(proposal.tx.prefix.fee, MINIMUM_FEE * 10);
    }

    // We should be able to create a transaction without any change outputs
    #[test_with_logger]
    fn test_no_change(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread =
            SyncThread::start(ledger_db.clone(), wallet_db.clone(), None, logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![70 * MOB as u64],
            &mut rng,
        );

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &wallet_db, &ledger_db, &mut rng, &logger);

        // Set value to consume the whole TXO and not produce change
        let value = 70 * MOB as u64 - MINIMUM_FEE;
        builder.add_recipient(recipient.clone(), value).unwrap();
        builder.select_txos(None).unwrap();
        builder.set_tombstone(0).unwrap();

        // Verify that not setting fee results in default fee
        let proposal = builder.build().unwrap();
        assert_eq!(proposal.tx.prefix.fee, MINIMUM_FEE);
        assert_eq!(proposal.outlays.len(), 1);
        assert_eq!(proposal.outlays[0].receiver, recipient);
        assert_eq!(proposal.outlays[0].value, value);
        assert_eq!(proposal.tx.prefix.inputs.len(), 1); // uses just one input
        assert_eq!(proposal.tx.prefix.outputs.len(), 1); // only one output to
                                                         // self (no change)
    }

    // We should be able to add multiple TxOuts to the same recipient, not to
    // multiple
    #[test_with_logger]
    fn test_add_multiple_outputs_to_same_recipient(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread =
            SyncThread::start(ledger_db.clone(), wallet_db.clone(), None, logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![70 * MOB as u64, 80 * MOB as u64, 90 * MOB as u64],
            &mut rng,
        );

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &wallet_db, &ledger_db, &mut rng, &logger);

        builder
            .add_recipient(recipient.clone(), 10 * MOB as u64)
            .unwrap();
        builder
            .add_recipient(recipient.clone(), 20 * MOB as u64)
            .unwrap();
        builder
            .add_recipient(recipient.clone(), 30 * MOB as u64)
            .unwrap();
        builder
            .add_recipient(recipient.clone(), 40 * MOB as u64)
            .unwrap();

        builder.select_txos(None).unwrap();
        builder.set_tombstone(0).unwrap();

        // Verify that not setting fee results in default fee
        let proposal = builder.build().unwrap();
        assert_eq!(proposal.tx.prefix.fee, MINIMUM_FEE);
        assert_eq!(proposal.outlays.len(), 4);
        assert_eq!(proposal.outlays[0].receiver, recipient);
        assert_eq!(proposal.outlays[0].value, 10 * MOB as u64);
        assert_eq!(proposal.outlays[1].receiver, recipient);
        assert_eq!(proposal.outlays[1].value, 20 * MOB as u64);
        assert_eq!(proposal.outlays[2].receiver, recipient);
        assert_eq!(proposal.outlays[2].value, 30 * MOB as u64);
        assert_eq!(proposal.outlays[3].receiver, recipient);
        assert_eq!(proposal.outlays[3].value, 40 * MOB as u64);
        assert_eq!(proposal.tx.prefix.inputs.len(), 2);
        assert_eq!(proposal.tx.prefix.outputs.len(), 5); // outlays + change
    }

    // Adding multiple values that exceed u64::MAX should fail
    #[test_with_logger]
    fn test_add_multiple_outputs_integer_overflow(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread =
            SyncThread::start(ledger_db.clone(), wallet_db.clone(), None, logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![
                7_000_000 * MOB as u64,
                7_000_000 * MOB as u64,
                7_000_000 * MOB as u64,
                7_000_000 * MOB as u64,
            ],
            &mut rng,
        );

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &wallet_db, &ledger_db, &mut rng, &logger);

        builder
            .add_recipient(recipient.clone(), 7_000_000 * MOB as u64)
            .unwrap();
        builder
            .add_recipient(recipient.clone(), 7_000_000 * MOB as u64)
            .unwrap();
        builder
            .add_recipient(recipient.clone(), 7_000_000 * MOB as u64)
            .unwrap();

        match builder.select_txos(None) {
            Ok(_) => panic!("Should not be able to select txos with > u64::MAX output value"),
            Err(WalletTransactionBuilderError::OutboundValueTooLarge) => {}
            Err(e) => panic!("Unexpected error {:?}", e),
        }
    }

    // We should be able to add multiple TxOuts to multiple recipients.
    #[test_with_logger]
    fn test_add_multiple_recipients(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread =
            SyncThread::start(ledger_db.clone(), wallet_db.clone(), None, logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![70 * MOB as u64, 80 * MOB as u64, 90 * MOB as u64],
            &mut rng,
        );

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &wallet_db, &ledger_db, &mut rng, &logger);

        builder
            .add_recipient(recipient.clone(), 10 * MOB as u64)
            .unwrap();

        // Create a new recipient
        let second_recipient = AccountKey::random(&mut rng).subaddress(0);
        builder
            .add_recipient(second_recipient.clone(), 40 * MOB as u64)
            .unwrap();
    }
}
