#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![warn(
    clippy::unwrap_used,
    missing_docs,
    rust_2018_idioms,
    unused_lifetimes,
    unused_qualifications
)]
#![doc = include_str!("../README.md")]

//***********************************************
// Re-Exports
//***********************************************

//-----------------------------------------------
// All Errors
//-----------------------------------------------
mod error;
#[doc(inline)]
pub use error::*;

//***********************************************
// (crate) Util
//***********************************************
mod util;
pub(crate) use util::*;

//-----------------------------------------------
// h2spec Server impl
//-----------------------------------------------
#[cfg(feature = "h2server")]
mod h2server;
#[cfg(feature = "h2server")]
#[doc(inline)]
pub use h2server::*;
