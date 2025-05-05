use std::fmt;
use std::sync::Arc;

use dom_query::{Document, NodeRef};
use html5ever::Attribute;
use tendril::StrTendril;

use super::builder::PluginPolicyBuilder;
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

/// A trait for sanitization directives. Defines methods for node and attribute sanitization.
pub trait SanitizePluginDirective {
    /// Sanitizes a node by removing elements and attributes according to the policy.
    ///
    /// - [Permissive] directive: Excludes elements (without their children) specified in the policy.
    ///   Removes element attributes specified in the policy.
    /// - [Restrictive] directive: Keeps only the elements and attributes explicitly allowed by the policy.
    ///
    /// May also remove elements excluded by the basic policy rules.
    fn sanitize_node(policy: &PluginPolicy<Self>, node: &NodeRef)
    where
        Self: Sized;

    /// Sanitizes the attributes of a node by removing or retaining them based on the policy.
    fn sanitize_node_attrs(policy: &PluginPolicy<Self>, node: &dom_query::NodeRef)
    where
        Self: Sized;
}

impl SanitizePluginDirective for Permissive {
    /// Sanitizes the descendants of a node.
    ///
    /// - Removes elements that **match** any of the [`PluginPolicy<T>::exclude_checkers`], but preserves their children.
    /// - Removes elements that match any of the [`PluginPolicy<T>::remove_checkers`], along with all their children.
    /// - Removes element attributes that match any of the [`PluginPolicy<T>::attr_exclude_checkers`].
    fn sanitize_node(policy: &PluginPolicy<Self>, node: &NodeRef) {
        if policy.exclude_checkers.is_empty()
            && policy.remove_checkers.is_empty()
            && policy.attr_exclude_checkers.is_empty()
        {
            return;
        }

        let mut child = node.first_child();

        while let Some(ref child_node) = child {
            let next_node = child_node.next_sibling();
            if policy.should_remove(child_node) {
                child_node.remove_from_parent();
                child = next_node;
                continue;
            }
            if child_node.may_have_children() {
                Self::sanitize_node(policy, child_node);
            }

            if policy.should_exclude(child_node) {
                if let Some(first_inline) = child_node.first_child() {
                    child_node.insert_siblings_before(&first_inline);
                };
                child_node.remove_from_parent();
            }
            Self::sanitize_node_attrs(policy, child_node);
            child = next_node;
        }
    }

    /// Removes element attributes that match any of the [`PluginPolicy<T>::attr_exclude_checkers`].
    fn sanitize_node_attrs(policy: &PluginPolicy<Self>, node: &dom_query::NodeRef) {
        if policy.attr_exclude_checkers.is_empty() {
            return;
        }

        policy.exclude_attrs(node, |node, attrs| node.remove_attrs(attrs));
    }
}

impl SanitizePluginDirective for Restrictive {
    /// Sanitizes the descendants of a node.
    ///
    /// - Removes elements that **do not match** any of the [`PluginPolicy<T>::exclude_checkers`], but preserves their children.
    /// - Removes elements that match any of the [`PluginPolicy<T>::remove_checkers`], along with all their children.
    /// - Removes all element attributes that **do not match** any of the [`PluginPolicy<T>::attr_exclude_checkers`].
    fn sanitize_node(policy: &PluginPolicy<Self>, node: &NodeRef) {
        let mut child = node.first_child();

        while let Some(ref child_node) = child {
            let next_node = child_node.next_sibling();

            if policy.should_remove(child_node) {
                child_node.remove_from_parent();
                child = next_node;
                continue;
            }
            if child_node.may_have_children() {
                policy.sanitize_node(child_node);
            }
            if !child_node.is_element() {
                child = next_node;
                continue;
            }
            if Self::should_skip(child_node) || policy.should_exclude(child_node) {
                Self::sanitize_node_attrs(policy, child_node);
                child = next_node;
                continue;
            }

            if let Some(first_inline) = child_node.first_child() {
                child_node.insert_siblings_before(&first_inline);
            };
            child_node.remove_from_parent();
            child = next_node;
        }
    }

    /// - Removes all element attributes that **do not match** any of the [`PluginPolicy<T>::attr_exclude_checkers`].
    fn sanitize_node_attrs(policy: &PluginPolicy<Self>, node: &dom_query::NodeRef) {
        if policy.attr_exclude_checkers.is_empty() {
            node.remove_all_attrs();
            return;
        }

        policy.exclude_attrs(node, |node, attrs| node.retain_attrs(attrs));
    }
}

/// A plugin based policy for sanitizing HTML documents.
#[derive(Clone)]
pub struct PluginPolicy<T: SanitizePluginDirective = Restrictive> {
    pub exclude_checkers: Arc<[Box<dyn NodeChecker>]>,
    pub remove_checkers: Arc<[Box<dyn NodeChecker>]>,
    pub attr_exclude_checkers: Arc<[Box<dyn AttrChecker>]>,
    pub(crate) _directive: std::marker::PhantomData<T>,
}

impl<T: SanitizePluginDirective> fmt::Debug for PluginPolicy<T> {
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

impl<T: SanitizePluginDirective> PluginPolicy<T> {
    fn should_exclude(&self, node: &NodeRef) -> bool {
        for checker in self.exclude_checkers.iter() {
            if checker.is_match(node) {
                return true;
            }
        }
        false
    }

    fn should_remove(&self, node: &NodeRef) -> bool {
        for checker in self.remove_checkers.iter() {
            if checker.is_match(node) {
                return true;
            }
        }
        false
    }
    fn should_exclude_attr(&self, node: &NodeRef, attr: &Attribute) -> bool {
        for checker in self.attr_exclude_checkers.iter() {
            if checker.is_match_attr(node, attr) {
                return true;
            }
        }
        false
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

impl<T: SanitizePluginDirective> PluginPolicy<T> {
    /// Creates a new [`PluginPolicyBuilder`] instance with the specified directive type.
    pub fn builder() -> PluginPolicyBuilder<T> {
        PluginPolicyBuilder::new()
    }
}

/// Alias for [`PluginPolicy`] using the [`Permissive`] directive (default-allow behavior).
pub type PermissivePluginPolicy<'a> = PluginPolicy<Permissive>;

/// Alias for [`PluginPolicy`] using the [`Restrictive`] directive (default-deny behavior).
pub type RestrictivePluginPolicy<'a> = PluginPolicy<Restrictive>;
