use std::time::Duration;

mod upnp;

pub use upnp::UpnpPortForwarder;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Protocol {
    TCP,
    UDP,
    Both
}

#[async_trait::async_trait]
pub trait PortForwarder {
    type Error: std::error::Error + Send + Sync;

    /// Try to open given port to the internet for given amount of time.
    /// 
    /// This method will return if the unforward was successful.
    async fn open(&self, port: u16, protocol: Protocol, duration: Duration) -> Result<bool, Self::Error>;

    /// Try to close given port from the internet.
    /// 
    /// This method will return if the unforward was successful.
    async fn close(&self, port: u16, protocol: Protocol) -> Result<bool, Self::Error>;

    /// Try to revert all the made port forwards.
    async fn discard(&self) -> Result<bool, Self::Error>;
}
