use crate::crypto::SecretKey;
use crate::rest_api::connect::ClientInfo;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Client {
    info: ClientInfo,
    secret_key: SecretKey
}

impl Client {
    #[inline]
    /// Build new client
    pub fn new(info: ClientInfo, secret_key: SecretKey) -> Self {
        Self {
            info,
            secret_key
        }
    }

    #[inline]
    /// Build new thin client
    pub fn thin(secret_key: SecretKey) -> Self {
        Self {
            info: ClientInfo::thin(),
            secret_key
        }
    }

    #[inline]
    /// Build new thin client with a random secret key
    pub fn random() -> Self {
        Self {
            info: ClientInfo::thin(),
            secret_key: SecretKey::random()
        }
    }

    #[inline]
    pub fn info(&self) -> &ClientInfo {
        &self.info
    }

    #[inline]
    pub fn secret_key(&self) -> &SecretKey {
        &self.secret_key
    }
}
