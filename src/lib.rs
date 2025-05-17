//! Flexible HTML sanitization for Rust — build policies and sanitize documents easily.

#![doc = include_str!("../Examples.md")]

pub mod directives;
mod dom_helpers;
pub mod plugin_policy;
pub mod policy;
pub mod traits;

pub(crate) mod macros;

#[doc(inline)]
pub use directives::{Permissive, Restrictive};
pub use policy::*;
