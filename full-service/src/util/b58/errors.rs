use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum B58Error {
    #[error("Invalid Printable Wrapper Type")]
    NotPrintableWrapper,

    #[error("Not A Public Address")]
    NotPublicAddress,

    #[error("Not A Payment Request")]
    NotPaymentRequest,

    #[error("Not A Transfer Payload")]
    NotTransferPayload,

    #[error("Transfer payload cannot have more than one entropy")]
    TransferPayloadRequiresSingleEntropy,

    #[error("Invalid Entropy")]
    InvalidEntropy,

    #[error("ProtoConversion: {0}")]
    ProtoConversion(mc_api::ConversionError),

    #[error("PrintableWrapper: {0}")]
    PrintableWrapper(mc_api::display::Error),
}

impl From<mc_api::ConversionError> for B58Error {
    fn from(src: mc_api::ConversionError) -> Self {
        Self::ProtoConversion(src)
    }
}

impl From<mc_api::display::Error> for B58Error {
    fn from(src: mc_api::display::Error) -> Self {
        Self::PrintableWrapper(src)
    }
}
