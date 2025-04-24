use super::core::{AttributeRule, Policy, SanitizeDirective};
use crate::Restrictive;

/// A builder for constructing a [`Policy`] with customizable sanitization rules.
///
/// The `PolicyBuilder` allows you to define rules for excluding specific elements or attributes
/// from the sanitization process. It supports both permissive and restrictive sanitization
/// directives, which determine how the exclusions are applied.
///
/// # Type Parameters
///
/// - `'a`: The lifetime of the references to elements and attributes.
/// - `T`: The sanitization directive, which must implement the [`SanitizeDirective`] trait.
///   Defaults to [`Restrictive`].
///
/// # Examples
///
/// ```rust
/// use dom_sanitizer::PolicyBuilder;
/// use dom_sanitizer::{Permissive, Restrictive};
///
/// let allow_policy = PolicyBuilder::<Permissive>::new()
///     .exclude_elements(&["nav"])
///     .exclude_attrs(&["onclick", "onload"])
///     .exclude_element_attrs("img", &["loading", "style"])
///     .remove_elements(&["script", "style"])
///     .build();
///
/// let deny_policy = PolicyBuilder::<Restrictive>::new()
///     .exclude_elements(&["p", "a", "span", "b", "i", "br"])
///     .exclude_attrs(&["id", "class", "role"])
///     .exclude_element_attrs("a", &["href", "target"])
///     .remove_elements(&["script", "style"])
///     .build();
/// ```
pub struct PolicyBuilder<'a, T: SanitizeDirective = Restrictive> {
    /// A list of rules for excluding attributes.
    attr_rules: Vec<AttributeRule<'a>>,
    /// A list of element names to exclude from the base policy.
    elements_to_exclude: Vec<&'a str>,
    /// The list of element names to be fully removed from the DOM tree, including their children.
    elements_to_remove: Vec<&'a str>,
    _directive: std::marker::PhantomData<T>,
}

impl<T: SanitizeDirective> Default for PolicyBuilder<'_, T> {
    fn default() -> Self {
        Self {
            attr_rules: vec![],
            elements_to_exclude: vec![],
            elements_to_remove: vec![],
            _directive: std::marker::PhantomData,
        }
    }
}

impl<'a, T: SanitizeDirective> PolicyBuilder<'a, T> {
    /// Creates a new [`PolicyBuilder`] with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Excludes the specified elements from the base sanitization directive.
    ///
    /// - If the sanitization directive is [`crate::Permissive`], these elements will be removed from the DOM.
    /// - If the sanitization directive is [`crate::Restrictive`], only these elements will be kept; all others will be removed.
    pub fn exclude_elements(mut self, elements: &'a [&str]) -> Self {
        self.elements_to_exclude.extend(elements);
        self
    }

    /// Specifies the names of elements to remove from the DOM with their children during sanitization.
    pub fn remove_elements(mut self, elements: &'a [&str]) -> Self {
        self.elements_to_remove.extend(elements);
        self
    }

    /// Excludes the specified attributes from the base sanitization directive.
    ///
    /// - If the sanitization directive is [`crate::Permissive`], these attributes will be removed from all elements where they appear.
    /// - If the sanitization directive is [`crate::Restrictive`], only these attributes will be kept; all others will be removed from all elements.
    pub fn exclude_attrs(mut self, attrs: &'a [&str]) -> Self {
        let rule = AttributeRule {
            element: None,
            attributes: attrs,
        };
        self.attr_rules.push(rule);
        self
    }

    /// Excludes the specified attributes from the base sanitization directive for a specific element.
    ///
    /// - If the sanitization directive is [`crate::Permissive`], these attributes will be removed from the specified element.
    /// - If the sanitization directive is [`crate::Restrictive`], only these attributes will be kept for the specified element; all others will be removed.
    pub fn exclude_element_attrs(mut self, element: &'a str, attrs: &'a [&str]) -> Self {
        let rule = AttributeRule {
            element: Some(element),
            attributes: attrs,
        };
        self.attr_rules.push(rule);
        self
    }

    /// Merges existing [`Policy`] into the builder, consuming it.
    pub fn merge(mut self, other: Policy<'a, T>) -> Self {
        self.attr_rules.extend(other.attr_rules);
        self.elements_to_exclude.extend(other.elements_to_exclude);
        self.elements_to_remove.extend(other.elements_to_remove);
        self
    }

    /// Builds the [`Policy`] using the current configuration.
    pub fn build(self) -> Policy<'a, T> {
        Policy {
            attr_rules: self.attr_rules,
            elements_to_exclude: self.elements_to_exclude,
            elements_to_remove: self.elements_to_remove,
            _directive: std::marker::PhantomData,
        }
    }
}
