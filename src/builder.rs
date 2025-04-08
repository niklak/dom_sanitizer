use crate::policy::{AttributeRule, Permissive, Policy, SanitizeDirective};

pub struct PolicyBuilder<'a, T: SanitizeDirective = Permissive> {
    pub attr_rules: Vec<AttributeRule<'a>>,
    pub element_rules: Vec<&'a str>,
    _directive: std::marker::PhantomData<T>,
}

impl<'a, T: SanitizeDirective> PolicyBuilder<'a, T> {
    /// Creates a new [`PolicyBuilder`] with default values.
    pub fn new() -> Self {
        Self {
            attr_rules: vec![],
            element_rules: vec![],
            _directive: std::marker::PhantomData,
        }
    }

    /// Excludes the given elements from the base sanitization directive.
    ///
    /// - If the sanitization directive is [`crate::Permissive`], these elements will be removed from the DOM.
    /// - If the sanitization directive is [`crate::Restrictive`], only these elements will be kept; all others will be removed.
    pub fn exclude_elements(mut self, elements: &'a [&str]) -> Self {
        self.element_rules.extend(elements);
        self
    }

    /// Excludes the given attributes from the base sanitization directive.
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

    /// Excludes the given attributes from the base sanitization directive for a specific element.
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
