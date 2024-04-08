// Copyright (c) 2020-2021 MobileCoin Inc.

//! A builder for transactions from the wallet. Note that we have a
//! TransactionBuilder in the MobileCoin transaction crate, but that is a lower
//! level of building, once you have already obtained all of the materials that
//! go into a transaction.
//!
//! This module, on the other hand, builds a transaction within the context of
//! the wallet.

use super::models::tx_proposal::{OutputTxo, UnsignedInputTxo, UnsignedTxProposal};
use crate::{
    db::{
        account::{AccountID, AccountModel},
        assigned_subaddress::AssignedSubaddressModel,
        models::{Account, Txo},
        txo::TxoModel,
        Conn,
    },
    error::WalletTransactionBuilderError,
    service::transaction::TransactionMemo,
};
use mc_account_keys::PublicAddress;
use mc_common::{logger::global_log, HashSet};
use mc_crypto_ring_signature_signer::OneTimeKeyDeriveData;
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::{Ledger, LedgerDB};
use mc_transaction_builder::{
    DefaultTxOutputsOrdering, EmptyMemoBuilder, InputCredentials, ReservedSubaddresses,
    TransactionBuilder,
};
use mc_transaction_core::{
    constants::RING_SIZE,
    tokens::Mob,
    tx::{TxOut, TxOutMembershipProof},
    Amount, BlockVersion, Token, TokenId,
};
use mc_util_uri::FogUri;
use rand::Rng;
use std::{collections::BTreeMap, str::FromStr, sync::Arc};

/// Default number of blocks used for calculating transaction tombstone block
/// number.
// TODO support for making this configurable
pub const DEFAULT_NEW_TX_BLOCK_ATTEMPTS: u64 = 10;

/// A builder of transactions constructed from this wallet.
pub struct WalletTransactionBuilder<FPR: FogPubkeyResolver + 'static> {
    /// Account ID (hex-encoded) from which to construct a transaction.
    account_id_hex: String,

    /// The ledger DB.
    ledger_db: LedgerDB,

    /// Optional inputs specified to use to construct the transaction.
    inputs: Vec<Txo>,

    /// Vector of (PublicAddress, Amounts) for the recipients of this
    /// transaction.
    outlays: Vec<(PublicAddress, u64, TokenId)>,

    /// The block after which this transaction is invalid.
    tombstone: u64,

    /// The fee for the transaction.
    fee: Option<(u64, TokenId)>,

    /// The block version for the transaction
    block_version: Option<BlockVersion>,

    /// Fog resolver maker, used when constructing outputs to fog recipients.
    /// This is abstracted because in tests, we don't want to form grpc
    /// connections to fog.
    #[allow(clippy::type_complexity)]
    fog_resolver_factory: Arc<dyn Fn(&[FogUri]) -> Result<FPR, String> + Send + Sync>,

    /// Subaddress (base58-encoded) from which to restrict TXOs for spending
    /// (optional)
    spend_only_from_subaddress: Option<String>,
}

impl<FPR: FogPubkeyResolver + 'static> WalletTransactionBuilder<FPR> {
    #[allow(clippy::type_complexity)]
    pub fn new(
        account_id_hex: String,
        ledger_db: LedgerDB,
        fog_resolver_factory: Arc<dyn Fn(&[FogUri]) -> Result<FPR, String> + Send + Sync + 'static>,
    ) -> Self {
        WalletTransactionBuilder {
            account_id_hex,
            ledger_db,
            inputs: vec![],
            outlays: vec![],
            tombstone: 0,
            fee: None,
            block_version: None,
            fog_resolver_factory,
            spend_only_from_subaddress: None,
        }
    }

    /// Sets the subaddress from which to restrict TXOs for spending.
    pub fn set_spend_only_from_subaddress(
        &mut self,
        subaddress: String
    ) -> Result<(), WalletTransactionBuilderError> {
        self.spend_only_from_subaddress = Some(subaddress);
        Ok(())
    }

    /// Sets inputs to the txos associated with the given txo_ids. Only unspent
    /// txos are included.
    pub fn set_txos(
        &mut self,
        conn: Conn,
        input_txo_ids: &[String],
    ) -> Result<(), WalletTransactionBuilderError> {
        let txos = Txo::select_by_id(input_txo_ids, conn)?;

        let unspent: Vec<Txo> = txos
            .iter()
            .filter(|txo| txo.spent_block_index.is_none())
            .cloned()
            .collect();

        self.inputs = unspent;

        Ok(())
    }

    /// Selects Txos from the account.
    pub fn select_txos(
        &mut self,
        conn: Conn,
        max_spendable_value: Option<u64>,
    ) -> Result<(), WalletTransactionBuilderError> {
        let mut outlay_value_sum_map: BTreeMap<TokenId, u128> =
            self.outlays
                .iter()
                .fold(BTreeMap::new(), |mut acc, (_, value, token_id)| {
                    acc.entry(*token_id)
                        .and_modify(|v| *v += *value as u128)
                        .or_insert(*value as u128);
                    acc
                });

        let (fee_value, fee_token_id) = self.fee.unwrap_or((Mob::MINIMUM_FEE, Mob::ID));
        outlay_value_sum_map
            .entry(fee_token_id)
            .and_modify(|v| *v += fee_value as u128)
            .or_insert(fee_value as u128);

        for (token_id, target_value) in outlay_value_sum_map {
            let fee_value = if token_id == fee_token_id {
                fee_value
            } else {
                0
            };

            self.inputs = Txo::select_spendable_txos_for_value(
                &self.account_id_hex,
                target_value,
                max_spendable_value,
                *token_id,
                fee_value,
                None, // FIXME: here's where we want to switch on the subaddress?
                conn,
            )?;
        }

        Ok(())
    }

    pub fn add_recipient(
        &mut self,
        recipient: PublicAddress,
        value: u64,
        token_id: TokenId,
    ) -> Result<(), WalletTransactionBuilderError> {
        self.outlays.push((recipient, value, token_id));
        Ok(())
    }

    pub fn set_fee(
        &mut self,
        fee: u64,
        token_id: TokenId,
    ) -> Result<(), WalletTransactionBuilderError> {
        if fee < 1 {
            return Err(WalletTransactionBuilderError::InsufficientFee(
                "1".to_string(),
            ));
        }
        self.fee = Some((fee, token_id));
        Ok(())
    }

    pub fn set_block_version(&mut self, block_version: BlockVersion) {
        self.block_version = Some(block_version);
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

    pub fn get_fog_resolver(&self, conn: Conn) -> Result<FPR, WalletTransactionBuilderError> {
        let account = Account::get(&AccountID(self.account_id_hex.clone()), conn)?;
        let change_subaddress = account.change_subaddress(conn)?;
        let change_public_address = change_subaddress.public_address()?;

        let fog_resolver = {
            let fog_uris = core::slice::from_ref(&change_public_address)
                .iter()
                .chain(self.outlays.iter().map(|(receiver, _, _)| receiver))
                .filter_map(|x| extract_fog_uri(x).transpose())
                .collect::<Result<Vec<_>, _>>()?;
            (self.fog_resolver_factory)(&fog_uris)
                .map_err(WalletTransactionBuilderError::FogPubkeyResolver)?
        };

        Ok(fog_resolver)
    }

    pub fn build(
        &self,
        memo: TransactionMemo,
        conn: Conn,
    ) -> Result<UnsignedTxProposal, WalletTransactionBuilderError> {
        let mut rng = rand::thread_rng();
        let account = Account::get(&AccountID(self.account_id_hex.clone()), conn)?;

        let view_account_key = account.view_account_key()?;
        let view_private_key = account.view_private_key()?;
        let reserved_subaddresses = ReservedSubaddresses::from(&view_account_key);

        let block_version = self.block_version.unwrap_or(BlockVersion::MAX);
        let (fee, fee_token_id) = self.fee.unwrap_or((Mob::MINIMUM_FEE, Mob::ID));
        let fee_amount = Amount::new(fee, fee_token_id);
        let fog_resolver = self.get_fog_resolver(conn)?;

        let memo_builder = match account.account_key() {
            Ok(account_key) => memo.memo_builder(&account_key),
            Err(_) => Box::<EmptyMemoBuilder>::default(),
        };

        let mut transaction_builder = TransactionBuilder::new_with_box(
            block_version,
            fee_amount,
            fog_resolver,
            memo_builder,
        )?;

        transaction_builder.set_tombstone_block(self.tombstone);

        if self.tombstone == 0 {
            return Err(WalletTransactionBuilderError::TombstoneNotSet);
        }

        if self.inputs.is_empty() {
            return Err(WalletTransactionBuilderError::NoInputs);
        }

        // Get membership proofs for our inputs
        let indexes = self
            .inputs
            .iter()
            .map(|utxo| {
                let txo = self.ledger_db.get_tx_out_by_index(
                    self.ledger_db
                        .get_tx_out_index_by_public_key(&utxo.public_key()?)?,
                )?;
                self.ledger_db.get_tx_out_index_by_hash(&txo.hash())
            })
            .collect::<Result<Vec<u64>, mc_ledger_db::Error>>()?;
        let proofs = self.ledger_db.get_tx_out_proof_of_memberships(&indexes)?;

        let inputs_and_proofs: Vec<(Txo, TxOutMembershipProof)> =
            self.inputs.clone().into_iter().zip(proofs).collect();

        let excluded_tx_out_indices: Vec<u64> = inputs_and_proofs
            .iter()
            .map(|(utxo, _membership_proof)| {
                let txo = self.ledger_db.get_tx_out_by_index(
                    self.ledger_db
                        .get_tx_out_index_by_public_key(&utxo.public_key()?)?,
                )?;
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

        let mut unsigned_input_txos = Vec::new();
        for (utxo, proof) in inputs_and_proofs.iter() {
            let subaddress_index = utxo.subaddress_index.ok_or_else(|| {
                WalletTransactionBuilderError::CannotUseOrphanedTxoAsInput(utxo.id.clone())
            })?;
            let db_tx_out = self.ledger_db.get_tx_out_by_index(
                self.ledger_db
                    .get_tx_out_index_by_public_key(&utxo.public_key()?)?,
            )?;

            let (mut ring, mut membership_proofs) = rings_and_proofs
                .pop()
                .ok_or(WalletTransactionBuilderError::RingsAndProofsEmpty)?;
            if ring.len() != membership_proofs.len() {
                return Err(WalletTransactionBuilderError::RingSizeMismatch);
            }

            // Add the input to the ring.
            let position_opt = ring.iter().position(|txo| *txo == db_tx_out);
            let real_index = match position_opt {
                Some(position) => {
                    // The input is already present in the ring.
                    // This could happen if ring elements are sampled
                    // randomly from the ledger.
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
                    // The real input is always the first element. This is
                    // safe because TransactionBuilder sorts each ring.
                    0
                }
            };

            if ring.len() != membership_proofs.len() {
                return Err(WalletTransactionBuilderError::RingSizeMismatch);
            }

            let onetime_key_derive_data =
                OneTimeKeyDeriveData::SubaddressIndex(subaddress_index as u64);

            let unsigned_input_txo = UnsignedInputTxo {
                tx_out: db_tx_out,
                subaddress_index: subaddress_index as u64,
                amount: Amount::new(utxo.value as u64, TokenId::from(utxo.token_id as u64)),
            };
            unsigned_input_txos.push(unsigned_input_txo);

            let input_credentials = InputCredentials::new(
                ring,
                membership_proofs,
                real_index,
                onetime_key_derive_data,
                view_private_key,
            )?;

            transaction_builder.add_input(input_credentials);
        }

        let mut total_value_per_token = BTreeMap::new();
        total_value_per_token.insert(fee_token_id, fee as u128);

        let mut payload_txos = Vec::new();
        for (receiver, amount, token_id) in self.outlays.clone().into_iter() {
            total_value_per_token
                .entry(token_id)
                .and_modify(|value| *value += amount as u128)
                .or_insert(amount as u128);

            let amount = Amount::new(amount, token_id);
            let tx_out_context = transaction_builder.add_output(amount, &receiver, &mut rng)?;

            let payload_txo = OutputTxo {
                tx_out: tx_out_context.tx_out,
                recipient_public_address: receiver,
                confirmation_number: tx_out_context.confirmation,
                amount,
                shared_secret: Some(tx_out_context.shared_secret),
            };
            payload_txos.push(payload_txo);
        }

        let input_value_per_token =
            inputs_and_proofs
                .iter()
                .fold(BTreeMap::new(), |mut acc, (utxo, _proof)| {
                    acc.entry(TokenId::from(utxo.token_id as u64))
                        .and_modify(|value| {
                            global_log::debug!(
                                "Adding value: {:?} to existing token: {:?}",
                                (utxo.value as u64) as u128,
                                *value
                            );
                            *value += (utxo.value as u64) as u128
                        })
                        .or_insert((utxo.value as u64) as u128);
                    acc
                });

        let mut change_txos = Vec::new();
        for (token_id, input_value) in input_value_per_token {
            let total_value = total_value_per_token.get(&token_id).ok_or_else(|| {
                WalletTransactionBuilderError::MissingInputsForTokenId(token_id.to_string())
            })?;

            if *total_value > input_value {
                return Err(WalletTransactionBuilderError::InsufficientInputFunds(format!(
                    "Total value required to send transaction {:?}, but only {:?} in inputs for token_id {:?}",
                    total_value,
                    input_value,
                    token_id.to_string(),
                )));
            }

            let change_value = input_value - *total_value;

            if change_value > u64::MAX as u128 {
                return Err(WalletTransactionBuilderError::ChangeLargerThanMaxValue(
                    change_value,
                ));
            }

            let change_amount = Amount::new(change_value as u64, token_id);
            let tx_out_context = transaction_builder.add_change_output(
                change_amount,
                &reserved_subaddresses,
                &mut rng,
            )?;

            let change_txo = OutputTxo {
                tx_out: tx_out_context.tx_out,
                recipient_public_address: reserved_subaddresses.change_subaddress.clone(),
                confirmation_number: tx_out_context.confirmation,
                amount: change_amount,
                shared_secret: Some(tx_out_context.shared_secret),
            };
            change_txos.push(change_txo);
        }

        let unsigned_tx = transaction_builder.build_unsigned::<DefaultTxOutputsOrdering>()?;

        Ok(UnsignedTxProposal {
            unsigned_tx,
            unsigned_input_txos,
            payload_txos,
            change_txos,
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
        let mut indexes_and_proofs_iterator = sampled_indices_vec.into_iter().zip(proofs);

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
    use std::ops::DerefMut;

    use super::*;
    use crate::{
        db::WalletDbError,
        service::sync::SyncThread,
        test_utils::{
            builder_for_random_recipient, get_test_ledger, random_account_with_seed_values,
            WalletDbTestContext, MOB,
        },
    };
    use mc_account_keys::AccountKey;
    use mc_common::logger::{async_test_with_logger, test_with_logger, Logger};
    use rand::{rngs::StdRng, SeedableRng};

    #[async_test_with_logger]
    async fn test_build_with_utxos(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread = SyncThread::start(ledger_db.clone(), wallet_db.clone(), logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &[11 * MOB, 11 * MOB, 11 * MOB, 111111 * MOB],
            &mut rng,
            &logger,
        );

        // Construct a transaction
        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();

        let account = Account::get(&AccountID::from(&account_key), conn).unwrap();

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng);

        // Send value specifically for your smallest Txo size. Should take 2 inputs
        // and also make change.
        let value = 11 * MOB;
        builder
            .add_recipient(recipient.clone(), value, Mob::ID)
            .unwrap();

        // Select the txos for the recipient
        builder.select_txos(conn, None).unwrap();
        builder.set_tombstone(0).unwrap();

        let unsigned_tx_proposal = builder
            .build(
                TransactionMemo::RTH {
                    subaddress_index: None,
                },
                conn,
            )
            .unwrap();
        let proposal = unsigned_tx_proposal.sign(&account).await.unwrap();
        assert_eq!(proposal.payload_txos.len(), 1);
        assert_eq!(proposal.payload_txos[0].recipient_public_address, recipient);
        assert_eq!(proposal.payload_txos[0].amount.value, value);
        assert_eq!(proposal.tx.prefix.inputs.len(), 2);
        assert_eq!(proposal.tx.prefix.fee, Mob::MINIMUM_FEE);
        assert_eq!(proposal.tx.prefix.outputs.len(), 2);
    }

    // Test that large values are handled correctly.
    #[test_with_logger]
    fn test_big_input_and_output_values(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 25, &mut rng);

        // Start sync thread
        let _sync_thread = SyncThread::start(ledger_db.clone(), wallet_db.clone(), logger.clone());

        // Give ourselves enough MOB that we have more than u64::MAX, 18_446_745 MOB
        // This is 55_000_000 * MOB
        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &[
                10_000_000 * MOB,
                9_000_000 * MOB,
                8_000_000 * MOB,
                7_000_000 * MOB,
                6_000_000 * MOB,
                5_000_000 * MOB,
                4_000_000 * MOB,
                3_000_000 * MOB,
                2_000_000 * MOB,
                1_000_000 * MOB,
            ],
            &mut rng,
            &logger,
        );

        // Check balance
        let unspent = Txo::list_unspent(
            Some(&AccountID::from(&account_key).to_string()),
            None,
            Some(0),
            None,
            None,
            None,
            None,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();
        let balance: u128 = unspent
            .iter()
            .map(|t| (t.value as u64) as u128)
            .sum::<u128>();
        assert_eq!(balance, 55_000_000 * MOB as u128);

        // Now try to send a transaction with a value (recipients + fee) > u64::MAX
        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng);

        let value = u64::MAX;
        builder.add_recipient(recipient, value, Mob::ID).unwrap();

        builder.set_tombstone(50).unwrap();

        // This should auto select the necessary txos correctly, which should be the
        // minimum set of (at most) 16 txos that sum to >= the total output value + fee
        builder.select_txos(conn, None).unwrap();

        let unsigned_tx_proposal = builder.build(TransactionMemo::Empty, conn).unwrap();

        // Check that the input txos are correct
        assert_eq!(unsigned_tx_proposal.unsigned_input_txos.len(), 6);
        assert_eq!(
            unsigned_tx_proposal.unsigned_input_txos[0].amount.value,
            1_000_000 * MOB
        );
        assert_eq!(
            unsigned_tx_proposal.unsigned_input_txos[1].amount.value,
            2_000_000 * MOB
        );
        assert_eq!(
            unsigned_tx_proposal.unsigned_input_txos[2].amount.value,
            3_000_000 * MOB
        );
        assert_eq!(
            unsigned_tx_proposal.unsigned_input_txos[3].amount.value,
            4_000_000 * MOB
        );
        assert_eq!(
            unsigned_tx_proposal.unsigned_input_txos[4].amount.value,
            5_000_000 * MOB
        );
        assert_eq!(
            unsigned_tx_proposal.unsigned_input_txos[5].amount.value,
            6_000_000 * MOB
        );

        // Check that the payload txos are correct
        assert_eq!(unsigned_tx_proposal.payload_txos.len(), 1);
        assert_eq!(unsigned_tx_proposal.payload_txos[0].amount.value, value);

        // Check that the change txo is correct
        assert_eq!(unsigned_tx_proposal.change_txos.len(), 1);
        assert_eq!(
            unsigned_tx_proposal.change_txos[0].amount.value,
            (21_000_000 * (MOB as u128) - value as u128 - (Mob::MINIMUM_FEE as u128)) as u64
        );
    }

    // Users should be able to set the txos specifically that they want to send
    #[async_test_with_logger]
    async fn test_setting_txos(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread = SyncThread::start(ledger_db.clone(), wallet_db.clone(), logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &[70 * MOB, 80 * MOB, 90 * MOB],
            &mut rng,
            &logger,
        );

        // Get our TXO list
        let txos: Vec<Txo> = Txo::list_for_account(
            &AccountID::from(&account_key).to_string(),
            None,
            None,
            None,
            None,
            None,
            Some(0),
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();

        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();

        let account = Account::get(&AccountID::from(&account_key), conn).unwrap();

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng);

        // Setting value to exactly the input will fail because you need funds for fee
        builder
            .add_recipient(recipient, txos[0].value as u64, Mob::ID)
            .unwrap();

        builder.set_txos(conn, &[txos[0].id.clone()]).unwrap();
        builder.set_tombstone(0).unwrap();
        match builder.build(
            TransactionMemo::RTH {
                subaddress_index: None,
            },
            conn,
        ) {
            Ok(_) => {
                panic!("Should not be able to construct Tx with > inputs value as output value")
            }
            Err(WalletTransactionBuilderError::InsufficientInputFunds(_)) => {}
            Err(e) => panic!("Unexpected error {:?}", e),
        }

        // Now build, setting to multiple TXOs
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng);

        // Set value to just slightly more than what fits in the one TXO
        builder
            .add_recipient(recipient.clone(), txos[0].value as u64 + 10, Mob::ID)
            .unwrap();

        builder
            .set_txos(conn, &[txos[0].id.clone(), txos[1].id.clone()])
            .unwrap();
        builder.set_tombstone(0).unwrap();
        let unsigned_tx_proposal = builder
            .build(
                TransactionMemo::RTH {
                    subaddress_index: None,
                },
                conn,
            )
            .unwrap();
        let proposal = unsigned_tx_proposal.sign(&account).await.unwrap();
        assert_eq!(proposal.payload_txos.len(), 1);
        assert_eq!(proposal.payload_txos[0].recipient_public_address, recipient);
        assert_eq!(
            proposal.payload_txos[0].amount.value,
            txos[0].value as u64 + 10
        );
        assert_eq!(proposal.tx.prefix.inputs.len(), 2); // need one more for fee
        assert_eq!(proposal.tx.prefix.fee, Mob::MINIMUM_FEE);
        assert_eq!(proposal.tx.prefix.outputs.len(), 2); // self and change
    }

    // This test is to ensure that we can send a transaction with a total input
    // value of > u64::MAX
    #[test_with_logger]
    fn test_setting_input_and_output_txos_overflow_u64(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let _sync_thread = SyncThread::start(ledger_db.clone(), wallet_db.clone(), logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &[u64::MAX, u64::MAX, u64::MAX],
            &mut rng,
            &logger,
        );

        let txos: Vec<Txo> = Txo::list_for_account(
            &AccountID::from(&account_key).to_string(),
            None,
            None,
            None,
            None,
            None,
            Some(0),
            conn,
        )
        .unwrap();

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng);

        // This will create a total recipient value of > u64::MAX, which should be valid
        builder
            .add_recipient(recipient.clone(), u64::MAX, Mob::ID)
            .unwrap();

        builder.add_recipient(recipient, u64::MAX, Mob::ID).unwrap();

        builder.set_tombstone(22).unwrap();
        builder.set_fee(Mob::MINIMUM_FEE, Mob::ID).unwrap();

        // This will create a total input value of > u64::MAX, which should be valid.
        builder
            .set_txos(
                conn,
                &[txos[0].id.clone(), txos[1].id.clone(), txos[2].id.clone()],
            )
            .unwrap();

        // NOTE: We have to use an Empty memo here because the RTH memo will fail
        // because of a u64::MAX limit on the total_outlay value.
        // See https://github.com/mobilecoinfoundation/mobilecoin/blob/437133a545b85958278efcb655bce36929c8f72a/transaction/extra/src/memo/destination_with_payment_request_id.rs#L51
        // for more details.
        let unsigned_tx_proposal = builder.build(TransactionMemo::Empty, conn).unwrap();

        // Check that the input txos are correct
        assert_eq!(unsigned_tx_proposal.unsigned_input_txos.len(), 3);
        assert_eq!(
            unsigned_tx_proposal.unsigned_input_txos[0].amount.value,
            u64::MAX
        );
        assert_eq!(
            unsigned_tx_proposal.unsigned_input_txos[1].amount.value,
            u64::MAX
        );
        assert_eq!(
            unsigned_tx_proposal.unsigned_input_txos[2].amount.value,
            u64::MAX
        );

        // Check that the payload txos are correct
        assert_eq!(unsigned_tx_proposal.payload_txos.len(), 2);
        assert_eq!(unsigned_tx_proposal.payload_txos[0].amount.value, u64::MAX);
        assert_eq!(unsigned_tx_proposal.payload_txos[1].amount.value, u64::MAX);

        // Check that the change txo is correct
        assert_eq!(unsigned_tx_proposal.change_txos.len(), 1);
        assert_eq!(
            unsigned_tx_proposal.change_txos[0].amount.value,
            u64::MAX - Mob::MINIMUM_FEE
        );
    }

    // This test is to check that change > u64::MAX is handled correctly. Currently,
    // we block this from happening, but we may want to allow it in the future by
    // automatically creating as many change txos as necessary within the 16 output
    // limit.
    //
    // For now, we will expect users to manually construct transactions within the
    // u64::MAX change limit if they run in to this.
    //
    // Note: This will not happen if txos are selected automatically since it only
    // searches incrementally for txos until it finds enough to cover the total
    // value, which should mean that it's never possible to be over the u64::MAX
    // limit.
    #[test_with_logger]
    fn test_change_over_u64max_fails(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let _sync_thread = SyncThread::start(ledger_db.clone(), wallet_db.clone(), logger.clone());

        // These are values close to u64::MAX, but easier to work with and test (quick
        // maths)
        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &[
                18000000000000000000,
                18000000000000000000,
                18000000000000000000,
            ],
            &mut rng,
            &logger,
        );

        let txos: Vec<Txo> = Txo::list_for_account(
            &AccountID::from(&account_key).to_string(),
            None,
            None,
            None,
            None,
            None,
            Some(0),
            conn,
        )
        .unwrap();

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng);

        // Adding a recipient that will cause the change to be > u64::MAX
        builder.add_recipient(recipient, 100, Mob::ID).unwrap();

        // Force setting this emulates a user manually setting the input txos. The
        // automatic txo selection may not necessarily select the same txos. This should
        // fail when trying to build.
        builder
            .set_txos(
                conn,
                &[txos[0].id.clone(), txos[1].id.clone(), txos[2].id.clone()],
            )
            .unwrap();

        builder.set_tombstone(22).unwrap();
        builder.set_fee(Mob::MINIMUM_FEE, Mob::ID).unwrap();

        // This should fail because the change will be equal to
        // 18000000000000000000 * 3 - 100 - 4000000000 = 53999999999599999900
        let err = builder
            .build(TransactionMemo::Empty, conn)
            .expect_err("Should fail");

        assert_eq!(
            err.to_string(),
            WalletTransactionBuilderError::ChangeLargerThanMaxValue(53999999999599999900)
                .to_string()
        )
    }

    // Test max_spendable correctly filters out txos above max_spendable
    #[async_test_with_logger]
    async fn test_max_spendable(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread = SyncThread::start(ledger_db.clone(), wallet_db.clone(), logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &[70 * MOB, 80 * MOB, 90 * MOB],
            &mut rng,
            &logger,
        );

        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();

        let account = Account::get(&AccountID::from(&account_key), conn).unwrap();

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng);

        // Setting value to exactly the input will fail because you need funds for fee
        builder
            .add_recipient(recipient.clone(), 80 * MOB, Mob::ID)
            .unwrap();

        // Test that selecting Txos with max_spendable < all our txo values fails
        match builder.select_txos(conn, Some(10)) {
            Ok(_) => panic!("Should not be able to construct tx when max_spendable < all txos"),
            Err(WalletTransactionBuilderError::WalletDb(WalletDbError::NoSpendableTxos(
                token_id,
            ))) => {
                assert_eq!(token_id, Mob::ID.to_string());
            }
            Err(e) => panic!("Unexpected error {:?}", e),
        }

        // We should be able to try again, with max_spendable at 70, but will not hit
        // our outlay target (80 * MOB)
        match builder.select_txos(conn, Some(70 * MOB)) {
            Ok(_) => panic!("Should not be able to construct tx when max_spendable < all txos"),
            Err(WalletTransactionBuilderError::WalletDb(
                WalletDbError::InsufficientFundsUnderMaxSpendable(_),
            )) => {}
            Err(e) => panic!("Unexpected error {:?}", e),
        }

        // Now, we should succeed if we set max_spendable = 80 * MOB, because we will
        // pick up both 70 and 80
        builder.select_txos(conn, Some(80 * MOB)).unwrap();
        builder.set_tombstone(0).unwrap();
        let unsigned_tx_proposal = builder
            .build(
                TransactionMemo::RTH {
                    subaddress_index: None,
                },
                conn,
            )
            .unwrap();
        let proposal = unsigned_tx_proposal.sign(&account).await.unwrap();
        assert_eq!(proposal.payload_txos.len(), 1);
        assert_eq!(proposal.payload_txos[0].recipient_public_address, recipient);
        assert_eq!(proposal.payload_txos[0].amount.value, 80 * MOB);
        assert_eq!(proposal.tx.prefix.inputs.len(), 2); // uses both 70 and 80
        assert_eq!(proposal.tx.prefix.fee, Mob::MINIMUM_FEE);
        assert_eq!(proposal.tx.prefix.outputs.len(), 2); // self and change
    }

    // Test setting and not setting tombstone block
    #[async_test_with_logger]
    async fn test_tombstone(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);
        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();

        // Start sync thread
        let _sync_thread = SyncThread::start(ledger_db.clone(), wallet_db.clone(), logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &[70 * MOB],
            &mut rng,
            &logger,
        );
        let account = Account::get(&AccountID::from(&account_key), conn).unwrap();

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng);

        builder.add_recipient(recipient, 10 * MOB, Mob::ID).unwrap();
        builder.select_txos(conn, None).unwrap();

        // Sanity check that our ledger is the height we think it is
        assert_eq!(ledger_db.num_blocks().unwrap(), 13);

        // We must set tombstone block before building
        match builder.build(
            TransactionMemo::RTH {
                subaddress_index: None,
            },
            conn,
        ) {
            Ok(_) => panic!("Expected TombstoneNotSet error"),
            Err(WalletTransactionBuilderError::TombstoneNotSet) => {}
            Err(e) => panic!("Unexpected error {:?}", e),
        }

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng);

        builder.add_recipient(recipient, 10 * MOB, Mob::ID).unwrap();
        builder.select_txos(conn, None).unwrap();

        // Set to default
        builder.set_tombstone(0).unwrap();

        // Not setting the tombstone results in tombstone = 0. This is an acceptable
        // value,
        let unsigned_tx_proposal = builder
            .build(
                TransactionMemo::RTH {
                    subaddress_index: None,
                },
                conn,
            )
            .unwrap();
        let proposal = unsigned_tx_proposal.sign(&account).await.unwrap();
        assert_eq!(proposal.tx.prefix.tombstone_block, 23);

        // Build a transaction and explicitly set tombstone
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng);

        builder.add_recipient(recipient, 10 * MOB, Mob::ID).unwrap();
        builder.select_txos(conn, None).unwrap();

        // Set to default
        builder.set_tombstone(20).unwrap();

        // Not setting the tombstone results in tombstone = 0. This is an acceptable
        // value,
        let unsigned_tx_proposal = builder
            .build(
                TransactionMemo::RTH {
                    subaddress_index: None,
                },
                conn,
            )
            .unwrap();
        let proposal = unsigned_tx_proposal.sign(&account).await.unwrap();
        assert_eq!(proposal.tx.prefix.tombstone_block, 20);
    }

    // Test setting and not setting the fee
    #[async_test_with_logger]
    async fn test_fee(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread = SyncThread::start(ledger_db.clone(), wallet_db.clone(), logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &[70 * MOB],
            &mut rng,
            &logger,
        );

        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();

        let account = Account::get(&AccountID::from(&account_key), conn).unwrap();

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng);

        builder.add_recipient(recipient, 10 * MOB, Mob::ID).unwrap();
        builder.select_txos(conn, None).unwrap();
        builder.set_tombstone(0).unwrap();

        // Verify that not setting fee results in default fee
        let unsigned_tx_proposal = builder
            .build(
                TransactionMemo::RTH {
                    subaddress_index: None,
                },
                conn,
            )
            .unwrap();
        let proposal = unsigned_tx_proposal.sign(&account).await.unwrap();
        assert_eq!(proposal.tx.prefix.fee, Mob::MINIMUM_FEE);

        // You cannot set fee to 0
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng);

        builder.add_recipient(recipient, 10 * MOB, Mob::ID).unwrap();
        builder.select_txos(conn, None).unwrap();
        builder.set_tombstone(0).unwrap();
        match builder.set_fee(0, Mob::ID) {
            Ok(_) => panic!("Should not be able to set fee to 0"),
            Err(WalletTransactionBuilderError::InsufficientFee(_)) => {}
            Err(e) => panic!("Unexpected error {:?}", e),
        }

        // Verify that not setting fee results in default fee
        let unsigned_tx_proposal = builder
            .build(
                TransactionMemo::RTH {
                    subaddress_index: None,
                },
                conn,
            )
            .unwrap();
        let proposal = unsigned_tx_proposal.sign(&account).await.unwrap();
        assert_eq!(proposal.tx.prefix.fee, Mob::MINIMUM_FEE);

        // Setting fee less than minimum fee should fail
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng);

        builder.add_recipient(recipient, 10 * MOB, Mob::ID).unwrap();
        builder.select_txos(conn, None).unwrap();
        builder.set_tombstone(0).unwrap();
        match builder.set_fee(0, Mob::ID) {
            Ok(_) => panic!("Should not be able to set fee to 0"),
            Err(WalletTransactionBuilderError::InsufficientFee(_)) => {}
            Err(e) => panic!("Unexpected error {:?}", e),
        }

        // Setting fee greater than MINIMUM_FEE works
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng);

        builder.add_recipient(recipient, 10 * MOB, Mob::ID).unwrap();
        builder.select_txos(conn, None).unwrap();
        builder.set_tombstone(0).unwrap();
        builder.set_fee(Mob::MINIMUM_FEE * 10, Mob::ID).unwrap();
        let unsigned_tx_proposal = builder
            .build(
                TransactionMemo::RTH {
                    subaddress_index: None,
                },
                conn,
            )
            .unwrap();
        let proposal = unsigned_tx_proposal.sign(&account).await.unwrap();
        assert_eq!(proposal.tx.prefix.fee, Mob::MINIMUM_FEE * 10);
    }

    // Even if change is zero, we should still have a change output
    #[async_test_with_logger]
    async fn test_change_zero_mob(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread = SyncThread::start(ledger_db.clone(), wallet_db.clone(), logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &[70 * MOB],
            &mut rng,
            &logger,
        );

        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng);

        // Set value to consume the whole TXO and not produce change
        let value = 70 * MOB - Mob::MINIMUM_FEE;
        builder
            .add_recipient(recipient.clone(), value, Mob::ID)
            .unwrap();
        builder.select_txos(conn, None).unwrap();
        builder.set_tombstone(0).unwrap();

        // Verify that not setting fee results in default fee
        let unsigned_tx_proposal = builder
            .build(
                TransactionMemo::RTH {
                    subaddress_index: None,
                },
                conn,
            )
            .unwrap();
        let account = Account::get(&AccountID::from(&account_key), conn).unwrap();
        let proposal = unsigned_tx_proposal.sign(&account).await.unwrap();

        assert_eq!(proposal.tx.prefix.fee, Mob::MINIMUM_FEE);
        assert_eq!(proposal.payload_txos.len(), 1);
        assert_eq!(proposal.payload_txos[0].recipient_public_address, recipient);
        assert_eq!(proposal.payload_txos[0].amount.value, value);
        assert_eq!(proposal.tx.prefix.inputs.len(), 1); // uses just one input
        assert_eq!(proposal.tx.prefix.outputs.len(), 2); // two outputs to
                                                         // self
    }

    // We should be able to add multiple TxOuts to the same recipient, not to
    // multiple
    #[async_test_with_logger]
    async fn test_add_multiple_outputs_to_same_recipient(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread = SyncThread::start(ledger_db.clone(), wallet_db.clone(), logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &[70 * MOB, 80 * MOB, 90 * MOB],
            &mut rng,
            &logger,
        );

        let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
        let conn = pooled_conn.deref_mut();
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng);

        builder
            .add_recipient(recipient.clone(), 10 * MOB, Mob::ID)
            .unwrap();
        builder
            .add_recipient(recipient.clone(), 20 * MOB, Mob::ID)
            .unwrap();
        builder
            .add_recipient(recipient.clone(), 30 * MOB, Mob::ID)
            .unwrap();
        builder
            .add_recipient(recipient.clone(), 40 * MOB, Mob::ID)
            .unwrap();

        builder.select_txos(conn, None).unwrap();
        builder.set_tombstone(0).unwrap();

        let unsigned_tx_proposal = builder
            .build(
                TransactionMemo::RTH {
                    subaddress_index: None,
                },
                conn,
            )
            .unwrap();
        let account = Account::get(&AccountID::from(&account_key), conn).unwrap();
        let proposal = unsigned_tx_proposal.sign(&account).await.unwrap();

        assert_eq!(proposal.tx.prefix.fee, Mob::MINIMUM_FEE);
        assert_eq!(proposal.payload_txos.len(), 4);
        assert_eq!(proposal.payload_txos[0].recipient_public_address, recipient);
        assert_eq!(proposal.payload_txos[0].amount.value, 10 * MOB);
        assert_eq!(proposal.payload_txos[1].recipient_public_address, recipient);
        assert_eq!(proposal.payload_txos[1].amount.value, 20 * MOB);
        assert_eq!(proposal.payload_txos[2].recipient_public_address, recipient);
        assert_eq!(proposal.payload_txos[2].amount.value, 30 * MOB);
        assert_eq!(proposal.payload_txos[3].recipient_public_address, recipient);
        assert_eq!(proposal.payload_txos[3].amount.value, 40 * MOB);
        assert_eq!(proposal.tx.prefix.inputs.len(), 2);
        assert_eq!(proposal.tx.prefix.outputs.len(), 5); // outlays + change
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
        let _sync_thread = SyncThread::start(ledger_db.clone(), wallet_db.clone(), logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &[70 * MOB, 80 * MOB, 90 * MOB],
            &mut rng,
            &logger,
        );

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng);

        builder.add_recipient(recipient, 10 * MOB, Mob::ID).unwrap();

        // Create a new recipient
        let second_recipient = AccountKey::random(&mut rng).subaddress(0);
        builder
            .add_recipient(second_recipient, 40 * MOB, Mob::ID)
            .unwrap();
    }
}
