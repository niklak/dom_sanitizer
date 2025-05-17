use dom_query::NodeRef;
use html5ever::LocalName;
use tendril::StrTendril;

use super::builder::PolicyBuilder;
use crate::macros::sanitize_methods;
use crate::traits::{SanitizeDirective, SanitizePolicy};
use crate::{Permissive, Restrictive};

fn is_node_name_in(names: &[LocalName], node: &NodeRef) -> bool {
    node.qual_name_ref()
        .map_or(false, |qual_name| names.contains(&qual_name.local))
}

/// An **excluding** rule for sanitizing attributes of a specific element.
#[derive(Debug, Clone, Default)]
pub(crate) struct AttributeRule<'a> {
    /// The name of the element to which this rule applies.
    /// If `None`, the rule applies to all elements.
    pub(crate) element: Option<LocalName>,
    /// The list of attribute keys to be excluded.
    pub(crate) attributes: &'a [&'a str],
}

#[derive(Debug, Clone)]
pub struct Policy<'a, T: SanitizeDirective = Restrictive> {
    /// The list of excluding rules for attributes.
    /// For [Permissive] directive: attributes to remove
    /// For [Restrictive] directive: attributes to keep
    pub(crate) attrs_to_exclude: Vec<AttributeRule<'a>>,
    /// The list of element names excluded from the base [Policy].
    /// For [Permissive] directive: elements to remove (keeping their children)
    /// For [Restrictive] directive: elements to keep
    pub(crate) elements_to_exclude: Vec<LocalName>,
    /// Specifies the names of elements to remove from the DOM with their children during sanitization.
    pub(crate) elements_to_remove: Vec<LocalName>,
    pub(crate) _directive: std::marker::PhantomData<T>,
}

impl<T: SanitizeDirective> Policy<'_, T> {
    sanitize_methods!();
}

impl<T: SanitizeDirective> SanitizePolicy for Policy<'_, T> {
    fn should_exclude(&self, node: &NodeRef) -> bool {
        is_node_name_in(&self.elements_to_exclude, node)
    }

    fn should_remove(&self, node: &NodeRef) -> bool {
        is_node_name_in(&self.elements_to_remove, node)
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
            if let Some(qual_name) = node.qual_name_ref() {
                for rule in &self.attrs_to_exclude {
                    let Some(element_name) = &rule.element else {
                        attrs.extend(rule.attributes);
                        continue;
                    };
                    if &qual_name.local == element_name {
                        attrs.extend(rule.attributes);
                    }
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
}

/// Alias for [`Policy`] using the [`Permissive`] directive (default-allow behavior).
pub type PermissivePolicy<'a> = Policy<'a, Permissive>;
/// Alias for [`PermissivePolicy`] — allows all elements and attributes by default.
pub type AllowAllPolicy<'a> = Policy<'a, Permissive>;

/// Alias for [`Policy`] using the [`Restrictive`] directive (default-deny behavior).
pub type RestrictivePolicy<'a> = Policy<'a, Restrictive>;
/// Alias for [`RestrictivePolicy`] — denies all elements and attributes by default.
pub type DenyAllPolicy<'a> = Policy<'a, Restrictive>;
