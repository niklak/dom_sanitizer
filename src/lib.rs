//! Flexible HTML sanitization for Rust — build policies and sanitize documents easily.

#![doc = include_str!("../Examples.md")]

pub mod plugin_policy;
pub mod policy;

pub use policy::*;

use dom_query::NodeRef;
use html5ever::local_name;

/// A base sanitization directive, which allows all elements and attributes,
/// excluding listed in policy.
#[derive(Debug, Clone, Copy)]
pub struct Permissive;

/// A base sanitization directive, which restricts all elements and attributes,
/// excluding listed in policy.
#[derive(Debug, Clone, Copy)]
pub struct Restrictive;

impl Restrictive {
    /// Checks if the node should be skipped during sanitization and never be removed.
    pub(crate) fn should_skip(node: &NodeRef) -> bool {
        node.qual_name_ref().map_or(false, |qual_name| {
            matches!(
                qual_name.local,
                local_name!("html") | local_name!("head") | local_name!("body")
            )
        })
    }
}
