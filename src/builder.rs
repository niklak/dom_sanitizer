use crate::policy::{AttributeRule, Permissive, Policy, SanitizeDirective};

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
///   Defaults to [`Permissive`].
///
/// # Examples
///
/// ```rust
/// use crate::builder::PolicyBuilder;
/// use crate::policy::{Permissive, Restrictive};
///
/// let allow_policy = PolicyBuilder::<Permissive>::new()
///     .exclude_elements(&["script", "style"])
///     .exclude_attrs(&["onclick", "onload"])
///     .exclude_element_attrs("img", &["loading", "style"])
///     .build();
///
/// let deny_policy = PolicyBuilder::<Restrictive>::new()
///     .exclude_elements(&["p", "a", "span", "b", "i", "br"])
///     .exclude_attrs(&["id", "class", "role"])
///     .exclude_element_attrs("a", &["href", "target"])
///     .build();
/// ```
pub struct PolicyBuilder<'a, T: SanitizeDirective = Permissive> {
    /// A list of rules for excluding attributes.
    pub attr_rules: Vec<AttributeRule<'a>>,
    /// A list of rules for excluding elements.
    pub element_rules: Vec<&'a str>,
    _directive: std::marker::PhantomData<T>,
}

impl<T: SanitizeDirective> Default for PolicyBuilder<'_, T> {
    fn default() -> Self {
        Self {
            attr_rules: vec![],
            element_rules: vec![],
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
        self.element_rules.extend(elements);
        self
    }

    /// Excludes the specified attributes from the base sanitization directive.
    ///
    /// - If the sanitization directive is [`crate::Permissive`], these attributes will be removed from all elements where they appear.
    /// - If the sanitization directive is [`crate::Restrictive`], only these attributes will be kept; all others will be removed from all elements.
    pub fn exclude_attrs(mut self, attrs: &'a [&str]) -> Self {
        let rule = AttributeRule {
            element: None,
            attributes: attrs.to_vec(),
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
            attributes: attrs.to_vec(),
        };
        self.attr_rules.push(rule);
        self
    }

    /// Builds the [`Policy`] using the current configuration.
    pub fn build(self) -> Policy<'a, T> {
        Policy {
            attr_rules: self.attr_rules,
            element_rules: self.element_rules,
            _directive: std::marker::PhantomData,
        }
    }
}
