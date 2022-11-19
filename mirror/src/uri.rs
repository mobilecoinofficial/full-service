// Copyright (c) 2018-2021 MobileCoin Inc.

use mc_util_uri::{Uri, UriScheme};

pub type WalletServiceMirrorUri = Uri<WalletServiceMirrorScheme>;

/// Wallet Service Mirror Uri Scheme
#[derive(Debug, Hash, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct WalletServiceMirrorScheme {}
impl UriScheme for WalletServiceMirrorScheme {
    /// The part before the '://' of a URL.
    const SCHEME_SECURE: &'static str = "wallet-service-mirror";
    const SCHEME_INSECURE: &'static str = "insecure-wallet-service-mirror";

    /// Default port numbers
    const DEFAULT_SECURE_PORT: u16 = 10443;
    const DEFAULT_INSECURE_PORT: u16 = 10080;
}
