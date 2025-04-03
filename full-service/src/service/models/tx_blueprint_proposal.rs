use super::{transaction_memo::TransactionMemo, tx_proposal::UnsignedInputTxo};
use mc_account_keys::PublicAddress;
use mc_transaction_builder::{TxBlueprint, TxOutContext};
use mc_transaction_types::Amount;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TxBlueprintProposalTxoContext {
    pub tx_out_context: TxOutContext,
    pub recipient_public_address: PublicAddress,
    pub amount: Amount,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TxBlueprintProposal {
    pub tx_blueprint: TxBlueprint,
    pub account_id_hex: String,
    pub memo: TransactionMemo,
    pub unsigned_input_txos: Vec<UnsignedInputTxo>,
    pub payload_txo_contexts: Vec<TxBlueprintProposalTxoContext>,
    pub change_txo_contexts: Vec<TxBlueprintProposalTxoContext>,
}
