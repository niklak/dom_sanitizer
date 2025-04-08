use dom_query::{Document, NodeRef};

use crate::builder::PolicyBuilder;

static ALWAYS_SKIP: &[&str] = &["html", "head", "body"];

//// A trait for sanitization directives, defines methods for node and attribute sanitization.
pub trait SanitizeDirective {
    fn sanitize_node(policy: &Policy<Self>, node: &NodeRef)
    where
        Self: Sized;
    fn sanitize_node_attr(policy: &Policy<Self>, node: &dom_query::NodeRef)
    where
        Self: Sized;
}

/// A base sanitization directive, which allows all elements and attributes,
/// excluding listed in policy.
#[derive(Debug, Clone, Copy)]
pub struct Permissive;

impl SanitizeDirective for Permissive {
    fn sanitize_node(policy: &Policy<Self>, node: &NodeRef) {
        if policy.element_rules.is_empty() && policy.attr_rules.is_empty() {
            return;
        }

        let mut child = node.first_child();

        while let Some(ref child_node) = child {
            let next_node = child_node.next_sibling();
            if child_node.may_have_children() {
                Self::sanitize_node(policy, child_node);
            }
            if !child_node.is_element() {
                child = next_node;
                continue;
            }
            if child_node.qual_name_ref().map_or(false, |name| {
                policy.element_rules.contains(&name.local.as_ref())
            }) {
                if let Some(first_inline) = child_node.first_child() {
                    child_node.insert_siblings_before(&first_inline);
                };
                child_node.remove_from_parent();
            }
            Self::sanitize_node_attr(policy, child_node);
            child = next_node;
        }
    }

    fn sanitize_node_attr(policy: &Policy<Self>, node: &dom_query::NodeRef) {
        if policy.attr_rules.is_empty() {
            return;
        }

        let attrs = policy.exclusive_attrs(node);
        node.remove_attrs(&attrs);
    }
}

/// A base sanitization directive, which restricts all elements and attributes,
/// excluding listed in policy.
#[derive(Debug, Clone, Copy)]
pub struct Restrictive;

impl SanitizeDirective for Restrictive {
    fn sanitize_node(policy: &Policy<Self>, node: &NodeRef) {
        if policy.element_rules.is_empty() && policy.attr_rules.is_empty() {
            return;
        }

        let mut child = node.first_child();

        while let Some(ref child_node) = child {
            let next_node = child_node.next_sibling();
            if child_node.may_have_children() {
                Self::sanitize_node(policy, child_node);
            }
            if !child_node.is_element() {
                child = next_node;
                continue;
            }
            if child_node.qual_name_ref().map_or(false, |name| {
                let local_name = name.local.as_ref();
                ALWAYS_SKIP.contains(&local_name) || policy.element_rules.contains(&local_name)
            }) {
                Self::sanitize_node_attr(policy, child_node);
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

    fn sanitize_node_attr(policy: &Policy<Self>, node: &dom_query::NodeRef) {
        if policy.attr_rules.is_empty() {
            node.remove_all_attrs();
            return;
        }

        let attrs = policy.exclusive_attrs(node);
        node.retain_attrs(&attrs);
    }
}

#[derive(Debug, Clone, Default)]
pub struct AttributeRule<'a> {
    pub element: Option<&'a str>,
    pub attributes: Vec<&'a str>,
}

#[derive(Debug, Clone)]
pub struct Policy<'a, T: SanitizeDirective = Permissive> {
    pub attr_rules: Vec<AttributeRule<'a>>,
    pub element_rules: Vec<&'a str>,
    pub(crate) _directive: std::marker::PhantomData<T>,
}

impl<'a, T: SanitizeDirective> Default for Policy<'a, T> {
    fn default() -> Self {
        Self {
            attr_rules: Vec::new(),
            element_rules: Vec::new(),
            _directive: std::marker::PhantomData,
        }
    }
}

impl<T: SanitizeDirective> Policy<'_, T> {
    pub fn sanitize_node(&self, node: &dom_query::NodeRef) {
        T::sanitize_node(self, node);
    }

    pub fn sanitize_document(&self, document: &Document) {
        self.sanitize_node(&document.root());
    }
}

impl<T: SanitizeDirective> Policy<'_, T> {
    fn exclusive_attrs(&self, node: &NodeRef) -> Vec<&str> {
        let Some(qual_name) = node.qual_name_ref() else {
            return vec![];
        };
        let mut attrs: Vec<&str> = vec![];

        for rule in &self.attr_rules {
            let Some(element_name) = rule.element else {
                attrs.extend(rule.attributes.iter());
                continue;
            };
            if qual_name.local.as_ref() == element_name {
                attrs.extend(rule.attributes.iter());
            }
        }
        attrs
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

pub type PermissivePolicy<'a> = Policy<'a, Permissive>;
pub type RestrictivePolicy<'a> = Policy<'a, Restrictive>;
