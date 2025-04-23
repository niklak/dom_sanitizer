use super::core::{AttrChecker, NodeChecker, PluginPolicy, SanitizePluginDirective};

use crate::Restrictive;

pub struct PluginPolicyBuilder<T: SanitizePluginDirective = Restrictive> {
    exclude_checkers: Vec<Box<dyn NodeChecker>>,
    remove_checkers: Vec<Box<dyn NodeChecker>>,
    attr_exclude_checkers: Vec<Box<dyn AttrChecker>>,
    _directive: std::marker::PhantomData<T>,
}
impl<T: SanitizePluginDirective> Default for PluginPolicyBuilder<T> {
    fn default() -> Self {
        Self {
            exclude_checkers: vec![],
            remove_checkers: vec![],
            attr_exclude_checkers: vec![],
            _directive: std::marker::PhantomData,
        }
    }
}

impl<T: SanitizePluginDirective> PluginPolicyBuilder<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn exclude<C: NodeChecker + 'static>(mut self, checker: C) -> Self {
        self.exclude_checkers.push(Box::new(checker));
        self
    }

    pub fn remove<C: NodeChecker + 'static>(mut self, checker: C) -> Self {
        self.remove_checkers.push(Box::new(checker));
        self
    }

    pub fn exclude_attr<C: AttrChecker + 'static>(mut self, checker: C) -> Self {
        self.attr_exclude_checkers.push(Box::new(checker));
        self
    }

    pub fn build(self) -> PluginPolicy<T> {
        PluginPolicy {
            exclude_checkers: self.exclude_checkers,
            remove_checkers: self.remove_checkers,
            attr_exclude_checkers: self.attr_exclude_checkers,
            _directive: std::marker::PhantomData,
        }
    }
}
