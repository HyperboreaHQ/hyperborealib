use std::path::PathBuf;

use serde_json::Value as Json;

use crate::time::timestamp;

use crate::crypto::prelude::*;
use crate::rest_api::prelude::*;

use super::MessagesInbox;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] AsJsonError),

    #[error(transparent)]
    Serialize(#[from] serde_json::Error)
}

#[derive(Debug, Clone)]
pub struct StoredQueueMessagesInbox {
    /// Path to the messages inbox's folder.
    pub storage_folder: PathBuf
}

impl StoredQueueMessagesInbox {
    pub async fn new(storage_folder: impl Into<PathBuf>) -> std::io::Result<Self> {
        let storage_folder = storage_folder.into();

        #[cfg(feature = "tracing")]
        tracing::trace!("Building new StoredQueueMessagesInbox in {:?}", storage_folder);

        tokio::fs::create_dir_all(&storage_folder).await?;

        Ok(Self {
            storage_folder
        })
    }
}

#[async_trait::async_trait]
impl MessagesInbox for StoredQueueMessagesInbox {
    type Error = Error;

    async fn add_message(
        &self,
        sender: Sender,
        receiver: PublicKey,
        channel: String,
        message: Message
    ) -> Result<(), Self::Error> {
        #[cfg(feature = "tracing")]
        tracing::debug!(
            sender = ?sender,
            receiver = receiver.to_base64(),
            channel,
            "Adding new message"
        );

        let folder = self.storage_folder
            .join(receiver.to_base64())
            .join(&channel);

        tokio::fs::create_dir_all(&folder).await?;

        let mut index = tokio::fs::read(folder.join("index")).await
            .unwrap_or(vec![]);

        let message_id = safe_random_u64();

        index.extend_from_slice(&message_id.to_be_bytes());

        let message_info = MessageInfo {
            sender,
            channel,
            message,
            received_at: timestamp()
        };

        let message_info = serde_json::to_vec(&message_info.to_json()?)?;

        tokio::fs::write(folder.join("index"), index).await?;
        tokio::fs::write(folder.join(message_id.to_string()), message_info).await?;

        Ok(())
    }

    async fn poll_messages(
        &self,
        receiver: PublicKey,
        channel: String,
        limit: Option<u64>
    ) -> Result<(Vec<MessageInfo>, u64), Self::Error> {
        #[cfg(feature = "tracing")]
        tracing::debug!(
            receiver = receiver.to_base64(),
            channel,
            limit,
            "Polling messages"
        );

        let folder = self.storage_folder
            .join(receiver.to_base64())
            .join(&channel);

        if let Ok(index) = tokio::fs::read(folder.join("index")).await {
            assert!(index.len() % 8 == 0);

            let mut bytes = [0; 8];
            let mut limit = limit.unwrap_or(u64::MAX);
            let mut shift = 0;

            let mut messages = Vec::new();

            for message_id in index.chunks(8) {
                if limit == 0 {
                    break;
                }

                bytes.copy_from_slice(message_id);

                let message_id = u64::from_be_bytes(bytes);
                let message_path = folder.join(message_id.to_string());

                if let Ok(message_info) = tokio::fs::read(&message_path).await {
                    let message_info = serde_json::from_slice::<Json>(&message_info)?;

                    messages.push(MessageInfo::from_json(&message_info)?);

                    limit -= 1;

                    tokio::fs::remove_file(message_path).await?;
                }

                shift += 8;
            }

            let index = &index[shift..];

            tokio::fs::write(folder.join("index"), index).await?;

            return Ok((
                messages,
                (index.len() / 8) as u64
            ));
        }

        Ok((vec![], 0))
    }
}

#[cfg(test)]
mod tests {
    use crate::rest_api::types::client::tests::get_client;
    use crate::rest_api::types::server::tests::get_server;

    use super::*;

    #[tokio::test]
    async fn send_poll() -> Result<(), Error> {
        let temp = std::env::temp_dir()
            .join("stored-queue-messages-inbox-test");

        if temp.exists() {
            tokio::fs::remove_dir_all(&temp).await?;
        }

        tokio::fs::create_dir(&temp).await?;

        let queue = StoredQueueMessagesInbox::new(&temp).await?;

        let sender_secret = SecretKey::random();
        let receiver_secret = SecretKey::random();

        let sender = Sender::new(get_client(), get_server());
        let receiver = get_client();

        let mut messages = Vec::with_capacity(5);

        for message in [b"message 1", b"message 2", b"message 3", b"message 4", b"message 5"] {
            let message = Message::create(
                &sender_secret,
                &receiver.public_key,
                message,
                MessageEncoding::default(),
                CompressionLevel::default()
            ).unwrap();

            messages.push(message.clone());

            queue.add_message(
                sender.clone(),
                receiver_secret.public_key(),
                String::from("default channel"),
                message
            ).await?;
        }

        assert_eq!(queue.poll_messages(receiver_secret.public_key(), String::from("random channel"), None).await?, (vec![], 0));
        assert_eq!(queue.poll_messages(receiver_secret.public_key(), String::from("random channel"), Some(100)).await?, (vec![], 0));

        let (poll, 4) = queue.poll_messages(receiver_secret.public_key(), String::from("default channel"), Some(1)).await? else {
            panic!("Test 1 failed");
        };

        assert_eq!(poll[0].message.read(&receiver_secret, &sender_secret.public_key()).unwrap(), b"message 1");

        let (poll, 2) = queue.poll_messages(receiver_secret.public_key(), String::from("default channel"), Some(2)).await? else {
            panic!("Test 2 failed");
        };

        assert_eq!(poll[0].message.read(&receiver_secret, &sender_secret.public_key()).unwrap(), b"message 2");
        assert_eq!(poll[1].message.read(&receiver_secret, &sender_secret.public_key()).unwrap(), b"message 3");

        let (poll, 0) = queue.poll_messages(receiver_secret.public_key(), String::from("default channel"), None).await? else {
            panic!("Test 3 failed");
        };

        assert_eq!(poll[0].message.read(&receiver_secret, &sender_secret.public_key()).unwrap(), b"message 4");
        assert_eq!(poll[1].message.read(&receiver_secret, &sender_secret.public_key()).unwrap(), b"message 5");

        Ok(())
    }
}
