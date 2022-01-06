//! Client library for [OpenRGB](https://openrgb.org) SDK server.
//!
//! This client is async and requires a [tokio](https://tokio.rs) runtime to run.
//!
//! # Example
//!
//! ```no_run
//! use openrgb::OpenRGB;
//! use std::error::Error;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!
//!     // connect to default server at localhost
//!     let client = OpenRGB::connect().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! See [examples](https://github.com/nicoulaj/openrgb-rs/tree/master/examples), and [OpenRGB] for client API.

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

#[macro_use]
extern crate enum_primitive_derive;
extern crate num_traits;

#[doc(inline)]
pub use {
    client::{DEFAULT_ADDR, DEFAULT_PROTOCOL, OpenRGB},
    error::OpenRGBError,
};

mod client;
mod error;
mod protocol;
pub mod data;

#[cfg(test)]
mod tests;
