//! Custom URI handler implementation.
//! 
//! This module is needed to properly handle
//! advanced URIs.
//! 
//! ## Supported URIs:
//! 
//! | Pattern                            | Meaning                  |
//! | ---------------------------------- | ------------------------ |
//! | `hyperborea://<public key>`        | Hyperborea thin client   |
//! | `hyperborea-client://<public key>` | Hyperborea thin client   |
//! | `hyperborea-server://<public key>` | Hyperborea server client |
//! | `hyperborea-file://<public key>  ` | Hyperborea file client   |
//! | `hyp://<public key>`               | Hyperborea thin client   |
//! | `hyp-client://<public key>`        | Hyperborea thin client   |
//! | `hyp-server://<public key>`        | Hyperborea server client |
//! | `hyp-file://<public key>`          | Hyperborea file client |
//! | `hyperborea://thin:<public key>`   | Hyperborea thin client   |
//! | `hyperborea://thick:<public key>`  | Hyperborea thick client  |
//! | `hyperborea://server:<public key>` | Hyperborea server client |
//! | `hyperborea://file:<public key>`   | Hyperborea file client   |
//! | `http://<address>`                 | HTTP server              |
//! | `https://<address>`                | HTTPS server             |

use std::str::FromStr;

use crate::crypto::prelude::*;
use crate::rest_api::types::ClientType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Address {
    /// Hyperborea client.
    /// 
    /// - `hyperborea://<public key>`
    /// - `hyperborea-client://<public key>`
    /// - `hyp-client://<public key>`
    /// - `hyperborea-server://<public key>`
    /// - `hyp-server://<public key>`
    /// - `hyperborea-file://<public key>`
    /// - `hyp-file://<public key>`
    Hyperborea {
        public_key: PublicKey,
        client_type: ClientType
    },

    /// HTTP server.
    /// 
    /// - `http://<address>`
    Http {
        address: String
    },

    /// HTTPS server.
    /// 
    /// - `https://<address>`
    Https {
        address: String
    },

    /// Raw address.
    /// 
    /// Stores unsupported value.
    Raw(String)
}

impl FromStr for Address {
    type Err = CryptographyError;

    fn from_str(address: &str) -> Result<Self, Self::Err> {
        let (protocol, address) = address
            .split_once("://")
            .unwrap_or(("", address));

        let address = address.to_owned();

        match protocol {
            "hyperborea" | "hyp" => {
                if let Some(address) = address.strip_prefix("thin:") {
                    Ok(Self::Hyperborea {
                        public_key: PublicKey::from_base64(address)?,
                        client_type: ClientType::Thin
                    })
                }

                else if let Some(address) = address.strip_prefix("thick:") {
                    Ok(Self::Hyperborea {
                        public_key: PublicKey::from_base64(address)?,
                        client_type: ClientType::Thick
                    })
                }

                else if let Some(address) = address.strip_prefix("server:") {
                    Ok(Self::Hyperborea {
                        public_key: PublicKey::from_base64(address)?,
                        client_type: ClientType::Server
                    })
                }

                else if let Some(address) = address.strip_prefix("file:") {
                    Ok(Self::Hyperborea {
                        public_key: PublicKey::from_base64(address)?,
                        client_type: ClientType::File
                    })
                }

                else {
                    Ok(Self::Hyperborea {
                        public_key: PublicKey::from_base64(&address)?,
                        client_type: ClientType::Thin
                    })
                }
            }

            "hyperborea-client" | "hyp-client" => Ok(Self::Hyperborea {
                public_key: PublicKey::from_base64(&address)?,
                client_type: ClientType::Thin
            }),

            "hyperborea-server" | "hyp-server" => Ok(Self::Hyperborea {
                public_key: PublicKey::from_base64(&address)?,
                client_type: ClientType::Server
            }),

            "hyperborea-file" | "hyp-file" => Ok(Self::Hyperborea {
                public_key: PublicKey::from_base64(&address)?,
                client_type: ClientType::File
            }),

            "http" => Ok(Self::Http {
                address
            }),

            "https" => Ok(Self::Https {
                address
            }),

            _ => Ok(Self::Raw(address))
        }
    }
}

#[inline]
/// Parse address info from the given URI.
/// 
/// This function calls `Address::from_str`.
pub fn parse(uri: impl AsRef<str>) -> Result<Address, CryptographyError> {
    Address::from_str(uri.as_ref())
}

#[cfg(test)]
mod tests {
    use super::{
        parse as parse_uri,
        *
    };

    #[test]
    fn parse() -> Result<(), CryptographyError> {
        let public_key = SecretKey::random().public_key();

        for protocol in ["hyperborea", "hyp"] {
            assert_eq!(parse_uri(format!("{protocol}://{}", public_key.to_base64()))?, Address::Hyperborea {
                public_key: public_key.clone(),
                client_type: ClientType::Thin
            });
        }

        for protocol in ["hyperborea", "hyp"] {
            for client_type in ["thin", "thick", "server", "file"] {
                assert_eq!(parse_uri(format!("{protocol}://{client_type}:{}", public_key.to_base64()))?, Address::Hyperborea {
                    public_key: public_key.clone(),
                    client_type: ClientType::from_str(client_type).unwrap()
                });
            }
        }

        for protocol in ["hyperborea-client", "hyp-client"] {
            assert_eq!(parse_uri(format!("{protocol}://{}", public_key.to_base64()))?, Address::Hyperborea {
                public_key: public_key.clone(),
                client_type: ClientType::Thin
            });
        }

        for protocol in ["hyperborea-server", "hyp-server"] {
            assert_eq!(parse_uri(format!("{protocol}://{}", public_key.to_base64()))?, Address::Hyperborea {
                public_key: public_key.clone(),
                client_type: ClientType::Server
            });
        }

        for protocol in ["hyperborea-file", "hyp-file"] {
            assert_eq!(parse_uri(format!("{protocol}://{}", public_key.to_base64()))?, Address::Hyperborea {
                public_key: public_key.clone(),
                client_type: ClientType::File
            });
        }

        assert_eq!(parse_uri("http://example.org")?, Address::Http {
            address: String::from("example.org")
        });

        assert_eq!(parse_uri("https://example.org")?, Address::Https {
            address: String::from("example.org")
        });

        assert_eq!(parse_uri("example.org")?, Address::Raw(String::from("example.org")));

        Ok(())
    }
}
