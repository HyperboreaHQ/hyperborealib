use std::sync::Mutex;
use std::collections::HashMap;

use easy_upnp::*;

use super::*;

#[derive(Debug)]
pub struct UpnpPortForwarder {
    forwarded: Mutex<HashMap<(u16, Protocol), UpnpConfig>>
}

impl UpnpPortForwarder {
    fn build_config(port: u16, protocol: PortMappingProtocol, duration: Duration) -> UpnpConfig {
        UpnpConfig {
            address: None,
            port,
            protocol,
            duration: duration.as_secs() as u32,
            comment: String::from("Hyperborea UPnP port forwarder")
        }
    }
}

#[async_trait::async_trait]
impl PortForwarder for UpnpPortForwarder {
    type Error = Error;

    async fn open(&self, port: u16, protocol: Protocol, duration: Duration) -> Result<bool, Self::Error> {
        let upnp_protocol = match protocol {
            Protocol::TCP => PortMappingProtocol::TCP,
            Protocol::UDP => PortMappingProtocol::UDP,

            Protocol::Both => return Ok(self.open(port, Protocol::TCP, duration).await? && self.open(port, Protocol::UDP, duration).await?)
        };

        add_ports([Self::build_config(port, upnp_protocol, duration)])
            .next()
            .unwrap()?;

        self.forwarded.lock()
            .expect("Failed to get forwards table")
            .insert((port, protocol), Self::build_config(port, upnp_protocol, duration));

        Ok(true)
    }

    async fn close(&self, port: u16, protocol: Protocol) -> Result<bool, Self::Error> {
        let mut forwarded = self.forwarded.lock()
            .expect("Failed to get forwards table");

        if let Some(config) = forwarded.remove(&(port, protocol)) {
            delete_ports([config])
                .next()
                .unwrap()?;
        }

        Ok(true)
    }

    async fn discard(&self) -> Result<bool, Self::Error> {
        let configs = self.forwarded.lock()
            .expect("Failed to get forwards table")
            .drain()
            .map(|(_, v)| v)
            .collect::<Vec<_>>();

        delete_ports(configs)
            .collect::<Result<_, _>>()?;

        Ok(true)
    }
}
