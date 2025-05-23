use dom_query::NodeRef;

use html5ever::local_name;

use crate::dom_helpers::next_child_or_sibling;
use crate::traits::{SanitizeDirective, SanitizePolicy};

/// A base sanitization directive, which allows all elements and attributes,
/// excluding listed in policy.
#[derive(Debug, Clone, Copy)]
pub struct Permissive;

impl SanitizeDirective for Permissive {
    /// Removes matching elements from the DOM keeping their children.
    /// Removes matching attributes from the element node.
    fn sanitize_node(policy: &impl SanitizePolicy, node: &NodeRef) {
        if policy.is_empty() {
            return;
        }
        let mut next_node = node.first_element_child();
        while let Some(child) = next_node {
            if policy.should_remove(&child) {
                next_node = next_child_or_sibling(&child, true, node);
                child.remove_from_parent();
                continue;
            }

            next_node = next_child_or_sibling(&child, false, node);
            if !policy.should_exclude(&child) {
                Self::sanitize_node_attrs(policy, &child);
                continue;
            }

            if let Some(first_inline) = child.first_child() {
                child.insert_siblings_before(&first_inline);
            };
            child.remove_from_parent();
        }
    }

    /// Removes matching attributes from the element node.
    fn sanitize_node_attrs(policy: &impl SanitizePolicy, node: &dom_query::NodeRef) {
        if !policy.has_attrs_to_exclude() {
            return;
        }
        policy.exclude_attrs(node, |node, attrs| node.remove_attrs(attrs));
    }
}

/// A base sanitization directive, which restricts all elements and attributes,
/// excluding listed in policy.
#[derive(Debug, Clone, Copy)]
pub struct Restrictive;

impl Restrictive {
    /// Checks if the node should be skipped during sanitization and never be removed.
    fn should_skip(node: &NodeRef) -> bool {
        node.qual_name_ref().map_or(false, |qual_name| {
            matches!(
                qual_name.local,
                local_name!("html") | local_name!("head") | local_name!("body")
            )
        })
    }
}

impl SanitizeDirective for Restrictive {
    /// Removes elements from the DOM keeping their children with exception of
    /// elements listed in policy.
    /// Removes attributes from the element node with exception of
    /// attributes listed in policy.
    fn sanitize_node(policy: &impl SanitizePolicy, node: &NodeRef) {
        let mut next_node = node.first_element_child();
        while let Some(child) = next_node {
            if policy.should_remove(&child) {
                next_node = next_child_or_sibling(&child, true, node);
                child.remove_from_parent();
                continue;
            }

            next_node = next_child_or_sibling(&child, false, node);

            if Self::should_skip(&child) || policy.should_exclude(&child) {
                Self::sanitize_node_attrs(policy, &child);
                continue;
            }

            if let Some(first_inline) = child.first_child() {
                child.insert_siblings_before(&first_inline);
            };
            child.remove_from_parent();
        }
    }

    /// Removes all attributes from the element node with exception of
    /// attributes listed in policy.
    fn sanitize_node_attrs(policy: &impl SanitizePolicy, node: &dom_query::NodeRef) {
        if !policy.has_attrs_to_exclude() {
            node.remove_all_attrs();
            return;
        }
        policy.exclude_attrs(node, |node, attrs| node.retain_attrs(attrs));
    }
}
