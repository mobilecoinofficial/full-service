use displaydoc::Display;

#[derive(Display, Debug)]
pub enum Error {
    /// GRPC: {0}
    Rpc(grpcio::Error),

    /// Api Conversion: {0}
    ApiConversion(mc_api::ConversionError),

    /// No reports returned from fog
    NoReports,
}

impl From<grpcio::Error> for Error {
    fn from(src: grpcio::Error) -> Self {
        Self::Rpc(src)
    }
}

impl From<mc_api::ConversionError> for Error {
    fn from(src: mc_api::ConversionError) -> Self {
        Self::ApiConversion(src)
    }
}

impl From<Error> for mc_connection::Error {
    fn from(src: Error) -> Self {
        match src {
            Error::Rpc(src) => mc_connection::Error::Grpc(src),
            Error::ApiConversion(src) => mc_connection::Error::Conversion(src),
            Error::NoReports => {
                mc_connection::Error::Other("No reports returned from fog".to_string())
            }
        }
    }
}
