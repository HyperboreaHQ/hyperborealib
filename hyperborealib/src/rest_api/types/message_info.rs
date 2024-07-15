use serde_json::{json, Value as Json};

use crate::rest_api::prelude::*;

use crate::time::timestamp;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MessageInfo {
    pub sender: Sender,
    pub channel: String,
    pub message: Message,
    pub received_at: u64
}

impl MessageInfo {
    #[inline]
    pub fn new(sender: Sender, channel: impl ToString, message: Message, received_at: u64) -> Self {
        Self {
            sender,
            channel: channel.to_string(),
            message,
            received_at
        }
    }

    #[inline]
    /// Run `new()` method with current timestamp.
    pub fn new_now(sender: Sender, channel: impl ToString, message: Message) -> Self {
        Self::new(sender, channel, message, timestamp())
    }
}

impl AsJson for MessageInfo {
    fn to_json(&self) -> Result<serde_json::Value, AsJsonError> {
        Ok(json!({
            "sender": self.sender.to_json()?,
            "channel": self.channel,
            "message": self.message.to_json()?,
            "received_at": self.received_at
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            sender: json.get("sender")
                .map(Sender::from_json)
                .ok_or_else(|| AsJsonError::FieldNotFound("sender"))??,

            channel: json.get("channel")
                .and_then(Json::as_str)
                .map(String::from)
                .ok_or_else(|| AsJsonError::FieldNotFound("channel"))?,

            message: json.get("message")
                .map(Message::from_json)
                .ok_or_else(|| AsJsonError::FieldNotFound("message"))??,

            received_at: json.get("received_at")
                .and_then(Json::as_u64)
                .ok_or_else(|| AsJsonError::FieldNotFound("received_at"))?
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::str::FromStr;

    use crate::rest_api::types::sender::tests::get_sender;

    use super::*;

    pub fn get_message_info() -> MessageInfo {
        let encoding = MessageEncoding::from_str("base64").unwrap();
        let message = Message::new("content", "sign", encoding);

        MessageInfo::new_now(get_sender(), "Hello, World!", message)
    }

    #[test]
    fn serialize() -> Result<(), AsJsonError> {
        let message_info = get_message_info();

        assert_eq!(MessageInfo::from_json(&message_info.to_json()?)?, message_info);

        Ok(())
    }
}
