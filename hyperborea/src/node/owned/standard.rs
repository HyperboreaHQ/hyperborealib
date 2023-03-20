use crate::node::Standard as RemoteStandard;
use super::Address;

/// `node::owned::Node` standard descriptor
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Standard {
    #[cfg(feature = "node-v1")]
    V1 {
        secret_key: k256::SecretKey
    }
}

impl Standard {
    #[cfg(feature = "node-v1")]
    #[inline]
    pub fn latest(secret_key: k256::SecretKey) -> Self {
        Self::V1 { secret_key }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        #[allow(unreachable_patterns)]
        match self {
            #[cfg(feature = "node-v1")]
            Standard::V1 { secret_key } => {
                let mut bytes = vec![0];

                bytes.extend_from_slice(secret_key.to_bytes().as_slice());

                bytes
            },

            _ => unreachable!()
        }
    }

    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> anyhow::Result<Self> {
        let bytes = bytes.as_ref();

        match bytes[0] {
            #[cfg(feature = "node-v1")]
            0 => Ok(Self::V1 {
                secret_key: k256::SecretKey::from_slice(&bytes[1..])?
            }),

            _ => anyhow::bail!("Unsupported `node::owned::Standard` bytes sequence found: {:?}", bytes)
        }
    }
}

impl<S: AsRef<Standard>> From<S> for RemoteStandard {
    #[inline]
    fn from(standard: S) -> Self {
        match standard.as_ref() {
            #[cfg(feature = "node-v1")]
            Standard::V1 { secret_key } => RemoteStandard::V1 { public_key: secret_key.public_key() }
        }
    }
}

impl From<&Standard> for Address {
    #[inline]
    fn from(standard: &Standard) -> Self {
        match standard {
            #[cfg(feature = "node-v1")]
            Standard::V1 { secret_key } => secret_key.public_key().into()
        }
    }
}

impl From<Standard> for Address {
    #[inline]
    fn from(standard: Standard) -> Self {
        Address::from(&standard)
    }
}

impl AsRef<Standard> for Standard {
    #[inline]
    fn as_ref(&self) -> &Standard {
        self
    }
}
