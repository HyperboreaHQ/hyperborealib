use serde_json::Value as Json;

use crate::crypto::prelude::*;
use crate::rest_api::prelude::*;

mod request;
mod response;

pub use request::DisconnectRequestBody;
pub use response::DisconnectResponseBody;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// `POST /api/v1/disconnect` request.
/// 
/// This request is sent to the `POST /api/v1/disconnect` to
/// tell a server that you want to disconnect from it.
/// 
/// This request is highly recommended to be sent automatically
/// when you close an application.
pub struct DisconnectRequest(pub Request<DisconnectRequestBody>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// `POST /api/v1/disconnect` response.
pub struct DisconnectResponse(pub Response<DisconnectResponseBody>);

impl DisconnectRequest {
    #[inline]
    /// Craft new `POST /api/v1/disconnect` client request.
    /// 
    /// - `client_secret` must contain reference to the
    ///   client's secret key. It is used to sign the proof
    ///   and connection certificate to the server.
    pub fn new(client_secret: &SecretKey) -> Self {
        Self(Request::new(client_secret, DisconnectRequestBody::new()))
    }

    #[inline]
    /// Validate the request.
    /// 
    /// Calls `validate()` function on the request's body
    /// and verifies that the provided connection certificate
    /// is signed for the specified server.
    pub fn validate(&self) -> Result<bool, ValidationError> {
        self.0.validate()
    }
}

impl AsJson for DisconnectRequest {
    #[inline]
    fn to_json(&self) -> Result<Json, AsJsonError> {
        self.0.to_json()
    }

    #[inline]
    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self(Request::from_json(json)?))
    }
}

impl DisconnectResponse {
    /// Create successful `POST /api/v1/announce` response.
    /// 
    /// - `status` must contain status code of the response
    ///   (`100 Success` in most cases).
    /// 
    /// - `server_secret` must contain reference to the
    ///   secret key of the responding server. It is used
    ///   to sign the response's proof.
    /// 
    /// - `proof_seed` must contain the same seed as used
    ///   in the original request.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use hyperborealib::crypto::prelude::*;
    /// use hyperborealib::rest_api::prelude::*;
    /// 
    /// let response = DisconnectResponse::success(
    ///     ResponseStatus::Success,
    ///     &SecretKey::random(),
    ///     safe_random_u64_long() // Here must be the original request's proof seed
    /// );
    /// ```
    pub fn success(status: ResponseStatus, server_secret: &SecretKey, proof_seed: u64) -> Self {
        let proof = server_secret.create_signature(proof_seed.to_be_bytes());

        Self(Response::success(
            status,
            server_secret.public_key(),
            proof,
            DisconnectResponseBody::new()
        ))
    }

    #[inline]
    /// Create failed `POST /api/v1/announce` response.
    /// 
    /// - `status` must contain response's status.
    /// 
    /// - `reason` must contain error reason (message and/or description).
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use hyperborealib::crypto::prelude::*;
    /// use hyperborealib::rest_api::prelude::*;
    /// 
    /// let response = DisconnectResponse::error(
    ///     ResponseStatus::ServerError,
    ///     "Example error"
    /// );
    /// ```
    pub fn error(status: ResponseStatus, reason: impl ToString) -> Self {
        Self(Response::error(status, reason))
    }

    #[inline]
    /// Validate the request.
    /// 
    /// Calls `validate()` function on the response's body.
    pub fn validate(&self, proof_seed: u64) -> Result<bool, ValidationError> {
        self.0.validate(proof_seed)
    }
}

impl AsJson for DisconnectResponse {
    #[inline]
    fn to_json(&self) -> Result<Json, AsJsonError> {
        self.0.to_json()
    }

    #[inline]
    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self(Response::from_json(json)?))
    }
}
