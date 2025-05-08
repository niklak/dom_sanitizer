use std::fmt;
use std::sync::Arc;

use dom_query::{Document, NodeRef};
use html5ever::Attribute;
use tendril::StrTendril;

use super::builder::PluginPolicyBuilder;
use crate::traits::{SanitizeDirective, SanitizePolicy};
use crate::{Permissive, Restrictive};

/// A trait for checking whether a node matches certain criteria.
///
/// This trait is used to determine whether a node should be excluded from a basic policy rule
/// or removed during the sanitization process. Implementors of this trait define the logic for
/// matching nodes based on specific conditions.
pub trait NodeChecker: Send + Sync {
    /// Returns `true` if the node is excluded by the basic policy or needs to be removed; otherwise, returns `false`.
    fn is_match(&self, _node: &NodeRef) -> bool;
}

/// A trait for checking whether an attribute matches certain criteria.
pub trait AttrChecker: Send + Sync {
    /// For [Permissive] directive, returning `true` means the attribute should be removed.
    /// For [Restrictive] directive, returning `true` means the attribute should be kept.
    fn is_match_attr(&self, _node: &NodeRef, _attr: &Attribute) -> bool;
}

/// A plugin based policy for sanitizing HTML documents.
#[derive(Clone)]
pub struct PluginPolicy<T: SanitizeDirective = Restrictive> {
    pub exclude_checkers: Arc<[Box<dyn NodeChecker>]>,
    pub remove_checkers: Arc<[Box<dyn NodeChecker>]>,
    pub attr_exclude_checkers: Arc<[Box<dyn AttrChecker>]>,
    pub(crate) _directive: std::marker::PhantomData<T>,
}

impl<T: SanitizeDirective> fmt::Debug for PluginPolicy<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PluginPolicy")
            .field(
                "exclude_checkers",
                &format_args!(
                    "Arc<[Box<dyn NodeChecker>]> ({} elements)",
                    self.exclude_checkers.len()
                ),
            )
            .field(
                "remove_checkers",
                &format_args!(
                    "Arc<[Box<dyn NodeChecker>]> ({} elements)",
                    self.remove_checkers.len()
                ),
            )
            .field(
                "attr_exclude_checkers",
                &format_args!(
                    "Arc<[Box<dyn AttrChecker>]> ({} elements)",
                    self.attr_exclude_checkers.len()
                ),
            )
            .field("_directive", &self._directive)
            .finish()
    }
}

impl<T: SanitizeDirective> SanitizePolicy for PluginPolicy<T> {
    fn should_exclude(&self, node: &NodeRef) -> bool {
        self.exclude_checkers
            .iter()
            .any(|checker| checker.is_match(node))
    }

    fn should_remove(&self, node: &NodeRef) -> bool {
        self.remove_checkers
            .iter()
            .any(|checker| checker.is_match(node))
    }

    fn has_attrs_to_exclude(&self) -> bool {
        !self.attr_exclude_checkers.is_empty()
    }

    fn exclude_attrs<F>(&self, node: &NodeRef, exclude_fn: F)
    where
        F: FnOnce(&NodeRef, &[&str]),
    {
        let node_attrs = node.attrs();
        let attrs: Vec<&str> = node_attrs
            .iter()
            .filter(|a| self.should_exclude_attr(node, a))
            .map(|a| a.name.local.as_ref())
            .collect();
        exclude_fn(node, &attrs)
    }

    fn is_empty(&self) -> bool {
        self.exclude_checkers.is_empty()
            && self.remove_checkers.is_empty()
            && self.attr_exclude_checkers.is_empty()
    }
}

impl<T: SanitizeDirective> PluginPolicy<T> {
    fn should_exclude_attr(&self, node: &NodeRef, attr: &Attribute) -> bool {
        for checker in self.attr_exclude_checkers.iter() {
            if checker.is_match_attr(node, attr) {
                return true;
            }
        }
        false
    }

    /// Sanitizes a node by applying the policy rules according to the directive type.
    ///
    /// For [Permissive] directive: Removes elements and attributes specified in the policy.
    /// For [Restrictive] directive: Keeps only elements and attributes specified in the policy.
    pub fn sanitize_node(&self, node: &NodeRef) {
        T::sanitize_node(self, node);
        node.normalize();
    }

    /// Sanitizes the document.
    pub fn sanitize_document(&self, document: &Document) {
        self.sanitize_node(&document.root());
    }

    /// Sanitizes the HTML content by applying the policy rules according to the directive type.
    pub fn sanitize_html<S: Into<StrTendril>>(&self, html: S) -> StrTendril {
        let doc = Document::from(html);
        self.sanitize_document(&doc);
        doc.html()
    }
}

impl<T: SanitizeDirective> PluginPolicy<T> {
    /// Creates a new [`PluginPolicyBuilder`] instance with the specified directive type.
    pub fn builder() -> PluginPolicyBuilder<T> {
        PluginPolicyBuilder::new()
    }
}

/// Alias for [`PluginPolicy`] using the [`Permissive`] directive (default-allow behavior).
pub type PermissivePluginPolicy<'a> = PluginPolicy<Permissive>;

/// Alias for [`PluginPolicy`] using the [`Restrictive`] directive (default-deny behavior).
pub type RestrictivePluginPolicy<'a> = PluginPolicy<Restrictive>;
