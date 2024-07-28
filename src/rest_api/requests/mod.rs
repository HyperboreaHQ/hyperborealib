//! Implementations of the requests and responses
//! to the protocol's REST API, described in the
//! paper. This module is intended to provide technical
//! support for data serialization and deserialization.
//! It is not intended to be used directly by general users.
//! It is recommended to make an abstraction over this
//! library and this particular module for easier use.
//! 
//! # Protocol architecture
//! 
//! ```text
//!  ┌────────┐                ┌────────┐ 
//!  │        │    ┌──────────►│        │ 
//!  │ Server │    │           │ Server │ 
//!  │        │◄───┼──────┐    │        │ 
//!  └────────┘    │      │    └────────┘ 
//!      ▲         │      │         ▲     
//!      │         │      │         │     
//!  ┌───┴────┐    │      │    ┌────┴───┐ 
//!  │        │    │      │    │        │ 
//!  │ Client ├────┘      └────┤ Client │ 
//!  │        │                │        │ 
//!  └────────┘                └────────┘ 
//! ```

mod clients;
mod servers;
mod info;
mod connect;
mod disconnect;
mod announce;
mod lookup;
mod send;
mod poll;

pub use clients::*;
pub use servers::*;
pub use info::*;
pub use connect::*;
pub use disconnect::*;
pub use announce::*;
pub use lookup::*;
pub use send::*;
pub use poll::*;
