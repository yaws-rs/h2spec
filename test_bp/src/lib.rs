#![warn(
    clippy::unwrap_used,
    missing_docs,
    rust_2018_idioms,
    unused_lifetimes,
    unused_qualifications
)]
#![doc = include_str!("../README.md")]

//***********************************************
// Required for testing
//***********************************************

#[cfg(test)]
mod ab_io_orbit;
#[doc(inline)]
#[cfg(test)]
pub(crate) use ab_io_orbit::*;

//-----------------------------------------------
// Rick Roll as a Test (RRaaT)
//-----------------------------------------------
#[cfg(test)]
mod t_rick_roll;
