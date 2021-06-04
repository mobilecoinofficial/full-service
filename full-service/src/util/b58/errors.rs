use displaydoc::Display;

#[derive(Display, Debug, PartialEq)]
pub enum B58Error {
    /// Invalid Printable Wrapper Type
    NotPrintableWrapper,

    /// Not A Public Address
    NotPublicAddress,

    /// Not A Payment Request
    NotPaymentRequest,

    /// Not A Transfer Payload
    NotTransferPayload,

    /// Transfer payload cannot have more than one entropy
    TransferPayloadRequiresSingleEntropy,

    /// Invalid Entropy
    InvalidEntropy,

    /// Proto Conversion
    ProtoConversion(mc_api::ConversionError),

    /// Printable Wrapper
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
