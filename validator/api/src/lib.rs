// Copyright (c) 2018-2022 MobileCoin, Inc.

//! Ledger Validator Node gRPC API.

use mc_util_uri::{Uri, UriScheme};

mod autogenerated_code {
    // Expose proto data types from included third-party/external proto files.
    pub use mc_api::{blockchain, external, printable};
    pub use mc_consensus_api::{consensus_common, consensus_common_grpc};
    pub use mc_fog_report_api::report;
    pub use protobuf::well_known_types::Empty;

    // Needed due to how to the auto-generated code references the Empty message.
    pub mod empty {
        pub use protobuf::well_known_types::Empty;
    }

    // Include the auto-generated code.
    include!(concat!(env!("OUT_DIR"), "/protos-auto-gen/mod.rs"));
}

pub use autogenerated_code::{validator_api::*, *};

pub type ValidatorUri = Uri<ValidatorScheme>;

/// Validator  Uri Scheme
#[derive(Debug, Hash, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct ValidatorScheme {}
impl UriScheme for ValidatorScheme {
    /// The part before the '://' of a URL.
    const SCHEME_SECURE: &'static str = "validator";
    const SCHEME_INSECURE: &'static str = "insecure-validator";

    /// Default port numbers
    const DEFAULT_SECURE_PORT: u16 = 5553;
    const DEFAULT_INSECURE_PORT: u16 = 5554;
}
