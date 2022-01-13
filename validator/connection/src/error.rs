use displaydoc::Display;

#[derive(Display, Debug)]
pub enum Error {
    /// GRPC: {0}
    Rpc(grpcio::Error),

    /// Api Conversion: {0}
    ApiConversion(mc_api::ConversionError),
}

impl From<grpcio::Error> for Error {
    fn from(src: grpcio::Error) -> Self {
        Self::Rpc(src)
    }
}

impl From<mc_api::ConversionError> for Error  {
    fn from(src: mc_api::ConversionError) -> Self {
        Self::ApiConversion(src)
    }
}

