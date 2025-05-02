//! Flexible HTML sanitization for Rust — build policies and sanitize documents easily.

#![doc= include_str!("../Examples.md")]

pub mod plugin_policy;
pub mod policy;

pub use policy::*;

/// A base sanitization directive, which allows all elements and attributes,
/// excluding listed in policy.
#[derive(Debug, Clone, Copy)]
pub struct Permissive;

/// A base sanitization directive, which restricts all elements and attributes,
/// excluding listed in policy.
#[derive(Debug, Clone, Copy)]
pub struct Restrictive;
