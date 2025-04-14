use dom_query::{Document, NodeRef};

use crate::{attr_parser::AttrMatcher, builder::PolicyBuilder};

/// Elements that should never be removed during sanitization, as they are
/// fundamental to the document structure.
static ALWAYS_SKIP: &[&str] = &["html", "head", "body"];

/// A trait for sanitization directives, defines methods for node and attribute sanitization.
pub trait SanitizeDirective {
    /// Sanitizes a node by removing elements and attributes based on the policy.
    fn sanitize_node(policy: &Policy<Self>, node: &NodeRef)
    where
        Self: Sized;
    /// Sanitizes the attributes of a node by removing or retaining them based on the policy.
    fn sanitize_node_attr(policy: &Policy<Self>, node: &dom_query::NodeRef)
    where
        Self: Sized;
}

/// A base sanitization directive, which allows all elements and attributes,
/// excluding listed in policy.
#[derive(Debug, Clone, Copy)]
pub struct Permissive;

impl SanitizeDirective for Permissive {
    /// Removes matching elements from the DOM keeping their children.
    /// Removes matching attributes from the element node.
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

    /// Removes matching attributes from the element node.
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
    /// Removes elements from the DOM keeping their children with exception of
    /// elements listed in policy.
    /// Removes attributes from the element node with exception of
    /// attributes listed in policy.
    fn sanitize_node(policy: &Policy<Self>, node: &NodeRef) {
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

    /// Removes all attributes from the element node with exception of
    /// attributes listed in policy.
    fn sanitize_node_attr(policy: &Policy<Self>, node: &dom_query::NodeRef) {
        if policy.attr_rules.is_empty() {
            node.remove_all_attrs();
            return;
        }

        let attrs = policy.exclusive_attrs(node);
        node.retain_attrs(&attrs);
    }
}

/// An **excluding** rule for sanitizing attributes of a specific element.
#[derive(Debug, Clone, Default)]
pub struct AttributeRule<'a> {
    /// The name of the element to which this rule applies.
    /// If `None`, the rule applies to all elements.
    pub element: Option<&'a str>,
    /// The list of attribute keys
    pub attributes: &'a [AttrMatcher<'a>],
}

#[derive(Debug, Clone)]
pub struct Policy<'a, T: SanitizeDirective = Permissive> {
    /// The list of excluding rules for attributes.
    /// For [Permissive] directive: attributes to remove
    /// For [Restrictive] directive: attributes to keep
    pub attr_rules: Vec<AttributeRule<'a>>,
    /// The list of excluding rules for elements.
    /// For [Permissive] directive: elements to remove
    /// For [Restrictive] directive: elements to keep
    pub element_rules: Vec<&'a str>,
    pub(crate) _directive: std::marker::PhantomData<T>,
}

impl<T: SanitizeDirective> Default for Policy<'_, T> {
    fn default() -> Self {
        Self {
            attr_rules: Vec::new(),
            element_rules: Vec::new(),
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
    /// Sanitizes the attributes of a node by applying the policy rules according to the directive type.
    pub fn sanitize_document(&self, document: &Document) {
        self.sanitize_node(&document.root());
    }
}

impl<T: SanitizeDirective> Policy<'_, T> {
    fn exclusive_attrs(&self, node: &NodeRef) -> Vec<&str> {
        let Some(qual_name) = node.qual_name_ref() else {
            return vec![];
        };
        let mut attrs_matchers: Vec<&AttrMatcher> = vec![];

        for rule in &self.attr_rules {
            let Some(element_name) = rule.element else {
                attrs_matchers.extend(rule.attributes.iter());
                continue;
            };
            if qual_name.local.as_ref() == element_name {
                attrs_matchers.extend(rule.attributes.iter());
            }
        }

        let Some(el) = node.element_ref() else {
            return vec![];
        };

        let mut exclusive_attrs: Vec<&str> = vec![];

        for matcher in attrs_matchers.iter() {
            let key = matcher.key;
            let is_match = match &matcher.value {
                Some(matcher_value) => el
                    .attrs
                    .iter()
                    .any(|a| &a.name.local == key && matcher_value.is_match(&a.value)),
                _ => el.has_attr(key),
            };
            
            if is_match {
                exclusive_attrs.push(key);
            }
        }
        exclusive_attrs
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
