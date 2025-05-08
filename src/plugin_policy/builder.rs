use std::sync::Arc;

use super::core::{AttrChecker, NodeChecker, PluginPolicy};
use crate::traits::SanitizeDirective;

use crate::Restrictive;

/// A builder for constructing a [`PluginPolicy`] with customizable sanitization rules.
///
/// The `PluginPolicyBuilder` allows you to define rules for excluding specific elements or attributes
/// from the sanitization process. It supports both permissive and restrictive sanitization
/// directives, which determine how the exclusions are applied.
///
/// # Type Parameters
///
/// - `T`: The sanitization directive, which must implement the [`SanitizeDirective`] trait. Defaults to [`Restrictive`].
///
/// # Example
/// ```
/// use html5ever::local_name;
/// use dom_sanitizer::plugin_policy::preset;
/// use dom_sanitizer::plugin_policy::{AttrChecker, PluginPolicy};
/// use dom_sanitizer::Restrictive;
///
/// use dom_query::NodeRef;
///
///struct SuspiciousAttr;
///impl AttrChecker for SuspiciousAttr {
///    fn is_match_attr(&self, _node: &NodeRef, attr: &html5ever::Attribute) -> bool {
///        let attr_name = attr.name.local.as_ref();
///        if attr_name != "onclick" && attr_name.starts_with("on") {
///            return true;
///        }
///        false
///    }
///}
///
///let policy: PluginPolicy<Restrictive> = PluginPolicy::builder()
///   .exclude_attr(SuspiciousAttr)
///   .remove(preset::LocalNamesMatcher::new(&["style", "script"]))
///   .build();
/// ```
pub struct PluginPolicyBuilder<T: SanitizeDirective = Restrictive> {
    exclude_checkers: Vec<Box<dyn NodeChecker>>,
    remove_checkers: Vec<Box<dyn NodeChecker>>,
    attr_exclude_checkers: Vec<Box<dyn AttrChecker>>,
    _directive: std::marker::PhantomData<T>,
}
impl<T: SanitizeDirective> Default for PluginPolicyBuilder<T> {
    fn default() -> Self {
        Self {
            exclude_checkers: vec![],
            remove_checkers: vec![],
            attr_exclude_checkers: vec![],
            _directive: std::marker::PhantomData,
        }
    }
}

impl<T: SanitizeDirective> PluginPolicyBuilder<T> {
    /// Creates a new `PluginPolicyBuilder` instance with default settings.
    pub fn new() -> Self {
        Self::default()
    }
    /// Creates a new `PluginPolicyBuilder` instance with the specified sanitization directive.
    pub fn exclude<C: NodeChecker + 'static>(mut self, checker: C) -> Self {
        self.exclude_checkers.push(Box::new(checker));
        self
    }
    /// Adds a node checker to the list of checkers that will be used to remove nodes.
    pub fn remove<C: NodeChecker + 'static>(mut self, checker: C) -> Self {
        self.remove_checkers.push(Box::new(checker));
        self
    }

    /// Adds an attribute checker to the list of checkers that will be used to exclude attributes from the base policy.
    pub fn exclude_attr<C: AttrChecker + 'static>(mut self, checker: C) -> Self {
        self.attr_exclude_checkers.push(Box::new(checker));
        self
    }

    pub fn build(self) -> PluginPolicy<T> {
        PluginPolicy {
            exclude_checkers: Arc::from(self.exclude_checkers),
            remove_checkers: Arc::from(self.remove_checkers),
            attr_exclude_checkers: Arc::from(self.attr_exclude_checkers),
            _directive: std::marker::PhantomData,
        }
    }
}
