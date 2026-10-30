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

//-----------------------------------------------
// H2spec Base Types
//-----------------------------------------------
mod h2types;
#[doc(inline)]
pub use h2types::*;

//-----------------------------------------------
// H2 App Traits
//-----------------------------------------------
mod t_h2app;
#[doc(inline)]
pub use t_h2app::*;

//-----------------------------------------------
// Internal Frame Formatter
//-----------------------------------------------
mod formatter;
pub(crate) use formatter::*;

//-----------------------------------------------
// Frame builder traits
//-----------------------------------------------
mod t_h2builder;
#[doc(inline)]
pub use t_h2builder::*;

//-----------------------------------------------
// H2spec Settings
//-----------------------------------------------
mod h2settings;
#[doc(inline)]
pub use h2settings::*;

//-----------------------------------------------
// Receiver (Parsing) Traits
//-----------------------------------------------
mod t_h2receivers;
#[doc(inline)]
pub use t_h2receivers::*;

//-----------------------------------------------
// Sender (Builder) Traits
//-----------------------------------------------
mod t_h2senders;
#[doc(inline)]
#[allow(unused_imports)]
pub use t_h2senders::*;

//-----------------------------------------------
// Parser impls
//-----------------------------------------------
mod h2parser;
#[doc(inline)]
pub use h2parser::*;

//-----------------------------------------------
// Validator impls - TODO
//-----------------------------------------------
mod h2validator;
#[doc(inline)]
#[allow(unused_imports)]
pub use h2validator::*;
