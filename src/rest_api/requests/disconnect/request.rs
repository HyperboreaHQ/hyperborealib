use serde_json::{json, Value as Json};

use crate::rest_api::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::large_enum_variant)]
/// `POST /api/v1/disconnect` request body.
/// 
/// Refer to the `DisconnectRequest` for details.
pub struct DisconnectRequestBody;

impl DisconnectRequestBody {
    #[inline]
    #[allow(clippy::new_without_default)]
    /// Create disconnect request body.
    /// 
    /// It doesn't contain any important info
    /// so everything is filled automatically.
    pub fn new() -> Self {
        Self
    }
}

impl AsJson for DisconnectRequestBody {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({}))
    }

    fn from_json(_json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() -> Result<(), AsJsonError> {
        let request = DisconnectRequestBody;

        assert_eq!(DisconnectRequestBody::from_json(&request.to_json()?)?, request);

        Ok(())
    }
}
