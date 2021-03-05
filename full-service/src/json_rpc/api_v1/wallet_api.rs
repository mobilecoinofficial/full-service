// Copyright (c) 2020-2021 MobileCoin Inc.

//! The Wallet API for Full Service. Version 1.

use crate::{
    json_rpc::api_v1::decorated_types::{JsonBlock, JsonBlockContents},
    service::WalletService,
};
use mc_common::logger::global_log;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_mobilecoind_json::data_types::{JsonTx, JsonTxOut};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

// Helper method to format displaydoc errors in json.
fn format_error<T: std::fmt::Display + std::fmt::Debug>(e: T) -> String {
    json!({"error": format!("{:?}", e), "details": e.to_string()}).to_string()
}

#[derive(Deserialize, Serialize, EnumIter, Debug)]
#[serde(tag = "method", content = "params")]
#[allow(non_camel_case_types)]
pub enum JsonCommandRequestV1 {
    get_transaction_object { transaction_log_id: String },
    get_txo_object { txo_id: String },
    get_block_object { block_index: String },
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "method", content = "result")]
#[allow(non_camel_case_types)]
#[allow(clippy::large_enum_variant)]
pub enum JsonCommandResponseV1 {
    get_transaction_object {
        transaction: JsonTx,
    },
    get_txo_object {
        txo: JsonTxOut,
    },
    get_block_object {
        block: JsonBlock,
        block_contents: JsonBlockContents,
    },
}

// The Wallet API inner method, which handles switching on the method enum.
//
// Note that this is structured this way so that the routes can be defined to
// take explicit Rocket state, and then pass the service to the inner method.
// This allows us to properly construct state with Mock Connection Objects in
// tests. This also allows us to version the overall API easily.
#[allow(clippy::bind_instead_of_map)]
pub fn wallet_api_inner_v1<T, FPR>(
    service: &WalletService<T, FPR>,
    command: Json<JsonCommandRequestV1>,
) -> Result<Json<JsonCommandResponseV1>, String>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    global_log::trace!("Running command {:?}", command);
    let result = match command.0 {
        JsonCommandRequestV1::get_transaction_object { transaction_log_id } => {
            JsonCommandResponseV1::get_transaction_object {
                transaction: service
                    .get_transaction_object(&transaction_log_id)
                    .map_err(format_error)?,
            }
        }
        JsonCommandRequestV1::get_txo_object { txo_id } => JsonCommandResponseV1::get_txo_object {
            txo: service.get_txo_object(&txo_id).map_err(format_error)?,
        },
        JsonCommandRequestV1::get_block_object { block_index } => {
            let (block, block_contents) = service
                .get_block_object(block_index.parse::<u64>().map_err(format_error)?)
                .map_err(format_error)?;
            JsonCommandResponseV1::get_block_object {
                block,
                block_contents,
            }
        }
    };
    Ok(Json(result))
}
