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
