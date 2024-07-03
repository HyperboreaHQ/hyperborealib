use serde_json::Value as Json;

use crate::crypto::Error as CryptographyError;

pub mod request;
pub mod response;
pub mod status;
pub mod middleware;

pub mod info;
pub mod clients;
pub mod servers;
pub mod connect;
pub mod lookup;
pub mod send;
pub mod poll;

pub mod prelude {
    pub use super::{
        AsJson,
        AsJsonError,
        ValidationError
    };

    pub use super::request::Request;
    pub use super::response::Response;
    pub use super::status::ResponseStatus;

    pub use super::middleware::{
        Client as ClientMiddleware,
        Server as ServerMiddleware,
        Error as MiddlewareError
    };

    pub use super::info::InfoResponse;

    pub use super::clients::{
        Client,
        ClientsResponse
    };

    pub use super::servers::{
        Server,
        ServersResponse
    };

    pub use super::connect::{
        ClientType,
        ClientInfo,
        ConnectionCertificate,
        ConnectionToken,
        ConnectRequest,
        ConnectRequestBody,
        ConnectResponse,
        ConnectResponseBody
    };

    pub use super::lookup::{
        LookupRequest,
        LookupRequestBody,
        LookupResponse,
        LookupResponseBody
    };

    pub use super::send::{
        TextEncoding,
        TextEncryption,
        TextCompression,
        MessageEncoding,
        Message,
        Sender,
        SendRequest,
        SendRequestBody,
        SendResponse,
        SendResponseBody,
        Error as SendError
    };

    pub use super::poll::{
        MessageInfo,
        PollRequest,
        PollRequestBody,
        PollResponse,
        PollResponseBody
    };
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Proof seed must be a 64 bit long unsigned integer")]
    InvalidSeed,

    #[error(transparent)]
    CryptographyError(#[from] CryptographyError)
}

#[derive(Debug, thiserror::Error)]
pub enum AsJsonError {
    #[error("Invalid standard version: {0}")]
    InvalidStandard(u64),

    #[error("Field `{0}` is not specified")]
    FieldNotFound(&'static str),

    #[error("Field `{0}` has invalid value")]
    FieldValueInvalid(&'static str),

    #[error(transparent)]
    Base64Error(#[from] base64::DecodeError),

    #[error(transparent)]
    CryptographyError(#[from] CryptographyError),

    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),

    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error>)
}

pub trait AsJson {
    fn to_json(&self) -> Result<Json, AsJsonError>;
    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized;
}

impl<T: AsJson> AsJson for Vec<T> {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        let values = self.iter()
            .map(T::to_json)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(serde_json::to_value(values)?)
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        let values = json.as_array()
            .ok_or_else(|| AsJsonError::Other("array expected".into()))?
            .iter()
            .map(T::from_json)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(values)
    }
}

impl<T: AsJson> AsJson for Box<T> {
    #[inline]
    fn to_json(&self) -> Result<Json, AsJsonError> {
        self.as_ref().to_json()
    }

    #[inline]
    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Box::new(T::from_json(json)?))
    }
}

#[macro_export]
/// Implement `AsJson` to the types with
/// implemented `serde::Serialize` and `serde::Deserialize`
/// traits.
macro_rules! impl_as_json {
    ($( $type:ty )*) => {
        $(
            impl $crate::rest_api::AsJson for $type {
                fn to_json(&self) -> Result<serde_json::Value, $crate::rest_api::AsJsonError> {
                    Ok(serde_json::to_value(())?)
                }

                fn from_json(json: &serde_json::Value) -> Result<Self, $crate::rest_api::AsJsonError> where Self: Sized {
                    Ok(serde_json::from_value(json.clone())?)
                }
            }
        )*
    }
}

impl_as_json!(
    ()
    u8 u16 u32 u64 u128 usize
    i8 i16 i32 i64 i128 isize
    String
    std::path::PathBuf
);
