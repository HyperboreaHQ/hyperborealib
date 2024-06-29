use serde_json::{json, Value as Json};

use crate::rest_api::{AsJson, AsJsonError};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConnectResponseBody;

impl ConnectResponseBody {
    #[inline]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }
}

impl AsJson for ConnectResponseBody {
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
        let response = ConnectResponseBody;

        assert_eq!(ConnectResponseBody::from_json(&response.to_json()?)?, response);

        Ok(())
    }
}
