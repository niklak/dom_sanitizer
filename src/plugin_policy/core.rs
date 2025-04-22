use dom_query::{Document, NodeRef};
use html5ever::Attribute;

use super::builder::PluginPolicyBuilder;

use crate::{Permissive, Restrictive};
pub trait NodeChecker {
    /// Checks if the node is allowed by the policy.
    fn is_match(&self, _node: &NodeRef) -> bool {
        // Default implementation allows all nodes.
        false
    }
}

pub trait AttrChecker {
    fn should_exclude_attr(&self, _node: &NodeRef, _attr: &Attribute) -> bool {
        false
    }
}

/// A trait for sanitization directives, defines methods for node and attribute sanitization.
pub trait SanitizePluginDirective {
    /// Sanitizes a node by removing elements and attributes based on the policy.
    fn sanitize_node(policy: &PluginPolicy<Self>, node: &NodeRef)
    where
        Self: Sized;
    /// Sanitizes the attributes of a node by removing or retaining them based on the policy.
    fn sanitize_node_attrs(policy: &PluginPolicy<Self>, node: &dom_query::NodeRef)
    where
        Self: Sized;
}

impl SanitizePluginDirective for Permissive {
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

    fn sanitize_node_attrs(policy: &PluginPolicy<Self>, node: &dom_query::NodeRef) {
        if policy.attr_exclude_checkers.is_empty() {
            return;
        }

        policy.exclude_attrs(node, |node, attrs| node.remove_attrs(attrs));
    }
}

impl SanitizePluginDirective for Restrictive {
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
            // TODO: Call should_remove_restrictive
            if policy.should_exclude(child_node) {
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

    fn sanitize_node_attrs(policy: &PluginPolicy<Self>, node: &dom_query::NodeRef) {
        if policy.attr_exclude_checkers.is_empty() {
            node.remove_all_attrs();
            return;
        }

        policy.exclude_attrs(node, |node, attrs| node.retain_attrs(attrs));
    }
}

pub struct PluginPolicy<T: SanitizePluginDirective = Restrictive> {
    pub exclude_checkers: Vec<Box<dyn NodeChecker>>,
    pub remove_checkers: Vec<Box<dyn NodeChecker>>,
    pub attr_exclude_checkers: Vec<Box<dyn AttrChecker>>,
    pub(crate) _directive: std::marker::PhantomData<T>,
}

impl<T: SanitizePluginDirective> PluginPolicy<T> {
    fn should_exclude(&self, node: &NodeRef) -> bool {
        for checker in &self.exclude_checkers {
            if checker.is_match(node) {
                return true;
            }
        }
        false
    }

    fn should_remove(&self, node: &NodeRef) -> bool {
        for checker in &self.remove_checkers {
            if checker.is_match(node) {
                return true;
            }
        }
        false
    }
    fn should_exclude_attr(&self, node: &NodeRef, attr: &Attribute) -> bool {
        for checker in &self.attr_exclude_checkers {
            if checker.should_exclude_attr(node, attr) {
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

    /// Sanitizes the attributes of a node by applying the policy rules according to the directive type.
    pub fn sanitize_document(&self, document: &Document) {
        self.sanitize_node(&document.root());
    }
}

impl<T: SanitizePluginDirective> PluginPolicy<T> {
    pub fn builder() -> PluginPolicyBuilder<T> {
        PluginPolicyBuilder::new()
    }
}
