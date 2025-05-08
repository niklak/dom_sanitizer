use dom_query::{Document, NodeRef};
use html5ever::LocalName;
use tendril::StrTendril;

use super::builder::PolicyBuilder;
use crate::{Permissive, Restrictive};

fn is_node_name_in(names: &[LocalName], node: &NodeRef) -> bool {
    let Some(qual_name) = node.qual_name_ref() else {
        return false;
    };
    names.contains(&qual_name.local)
}

/// A trait for sanitization directives, defines methods for node and attribute sanitization.
pub trait SanitizeDirective {
    /// Sanitizes a node by removing elements and attributes based on the policy.
    fn sanitize_node(policy: &impl SanitizePolicy, node: &NodeRef)
    where
        Self: Sized;
    /// Sanitizes the attributes of a node by removing or retaining them based on the policy.
    fn sanitize_node_attrs(policy: &impl SanitizePolicy, node: &dom_query::NodeRef)
    where
        Self: Sized;
}

impl SanitizeDirective for Permissive {
    /// Removes matching elements from the DOM keeping their children.
    /// Removes matching attributes from the element node.
    fn sanitize_node(policy: &impl SanitizePolicy, node: &NodeRef) {
        if policy.is_empty() {
            return;
        }
        let mut next_node = next_child_or_sibling(node, false);

        while let Some(child) = next_node {
            if policy.should_remove(&child) {
                next_node = next_child_or_sibling(&child, true);
                child.remove_from_parent();
                continue;
            }

            next_node = next_child_or_sibling(&child, false);
            if policy.should_exclude(&child) {
                if let Some(first_inline) = child.first_child() {
                    child.insert_siblings_before(&first_inline);
                };
                child.remove_from_parent();
            } else {
                Self::sanitize_node_attrs(policy, &child);
            }
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

impl SanitizeDirective for Restrictive {
    /// Removes elements from the DOM keeping their children with exception of
    /// elements listed in policy.
    /// Removes attributes from the element node with exception of
    /// attributes listed in policy.
    fn sanitize_node(policy: &impl SanitizePolicy, node: &NodeRef) {
        let mut next_node = next_child_or_sibling(node, false);
        while let Some(child) = next_node {
            
            if policy.should_remove(&child) {
                next_node = next_child_or_sibling(&child, true);
                child.remove_from_parent();
                continue;
            }
            
            next_node = next_child_or_sibling(&child, false);

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

/// An **excluding** rule for sanitizing attributes of a specific element.
#[derive(Debug, Clone, Default)]
pub struct AttributeRule<'a> {
    /// The name of the element to which this rule applies.
    /// If `None`, the rule applies to all elements.
    pub element: Option<LocalName>,
    /// The list of attribute keys to be excluded.
    pub attributes: &'a [&'a str],
}

#[derive(Debug, Clone)]
pub struct Policy<'a, T: SanitizeDirective = Restrictive> {
    /// The list of excluding rules for attributes.
    /// For [Permissive] directive: attributes to remove
    /// For [Restrictive] directive: attributes to keep
    pub attrs_to_exclude: Vec<AttributeRule<'a>>,
    /// The list of element names excluded from the base [Policy].
    /// For [Permissive] directive: elements to remove (keeping their children)
    /// For [Restrictive] directive: elements to keep
    pub elements_to_exclude: Vec<LocalName>,
    /// Specifies the names of elements to remove from the DOM with their children during sanitization.
    pub elements_to_remove: Vec<LocalName>,
    pub(crate) _directive: std::marker::PhantomData<T>,
}

impl<T: SanitizeDirective> Default for Policy<'_, T> {
    fn default() -> Self {
        Self {
            attrs_to_exclude: Vec::new(),
            elements_to_exclude: Vec::new(),
            elements_to_remove: Vec::new(),
            _directive: std::marker::PhantomData,
        }
    }
}

impl<T: SanitizeDirective> Policy<'_, T> {
    /// Sanitizes a node by applying the policy rules according to the directive type.
    ///
    /// For [Permissive] directive: Removes elements and attributes specified in the policy.
    /// For [Restrictive] directive: Keeps only elements and attributes specified in the policy.
    pub fn sanitize_node(&self, node: &dom_query::NodeRef) {
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

pub trait SanitizePolicy {
    fn should_exclude(&self, node: &NodeRef) -> bool;
    fn should_remove(&self, node: &NodeRef) -> bool;
    fn has_attrs_to_exclude(&self) -> bool;
    fn exclude_attrs<F>(&self, node: &NodeRef, exclude_fn: F) where  F: FnOnce(&NodeRef, &[&str]);
    fn is_empty(&self) -> bool { false }
}

impl <T: SanitizeDirective>SanitizePolicy for  Policy<'_, T> {
    fn should_exclude(&self, node: &NodeRef) -> bool {
        is_node_name_in(&self.elements_to_exclude, &node) 
    }

    fn should_remove(&self, node: &NodeRef) -> bool {
        is_node_name_in(&self.elements_to_remove, &node) 
    }

    fn has_attrs_to_exclude(&self) -> bool {
        !self.attrs_to_exclude.is_empty()
    }

    fn is_empty(&self) -> bool {
        self.elements_to_exclude.is_empty()
            && self.elements_to_remove.is_empty()
            && self.attrs_to_exclude.is_empty()
    }

    fn exclude_attrs<F>(&self, node: &NodeRef, exclude_fn: F)
    where
        F: FnOnce(&NodeRef, &[&str]),
    {
        let mut attrs: Vec<&str> = vec![];
        {
            let Some(qual_name) = node.qual_name_ref() else {
                return;
            };
    
            for rule in &self.attrs_to_exclude {
                let Some(element_name) = &rule.element else {
                    attrs.extend(rule.attributes.iter());
                    continue;
                };
                if &qual_name.local == element_name {
                    attrs.extend(rule.attributes.iter());
                }
            }
        }
        
        exclude_fn(node, &attrs)
    }
}

impl<'a, T: SanitizeDirective> Policy<'a, T> {
    /// Creates a new [`PolicyBuilder`] with default values.
    pub fn builder() -> PolicyBuilder<'a, T> {
        PolicyBuilder::new()
    }

    /// Creates a new [`Policy`] with default values.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Alias for [`Policy`] using the [`Permissive`] directive (default-allow behavior).
pub type PermissivePolicy<'a> = Policy<'a, Permissive>;
/// Alias for [`PermissivePolicy`] — allows all elements and attributes by default.
pub type AllowAllPolicy<'a> = Policy<'a, Permissive>;

/// Alias for [`Policy`] using the [`Restrictive`] directive (default-deny behavior).
pub type RestrictivePolicy<'a> = Policy<'a, Restrictive>;
/// Alias for [`RestrictivePolicy`] — denies all elements and attributes by default.
pub type DenyAllPolicy<'a> = Policy<'a, Restrictive>;

fn next_child_or_sibling<'a>(node: &NodeRef<'a>, ignore_child: bool) -> Option<NodeRef<'a>> {
    if !ignore_child {
        if let Some(first_child) = node.first_element_child() {
            return Some(first_child);
        }
    }

    if let Some(sibling) = node.next_element_sibling() {
        return Some(sibling);
    } 
    let mut parent = node.parent();
    while let Some(parent_node) = parent {
        if let Some(next_sibling) = parent_node.next_element_sibling() {
            return Some(next_sibling);
        } else {
            parent = parent_node.parent()
        }
    }
    None
}
