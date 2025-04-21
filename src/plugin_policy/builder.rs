use super::core::{
    AttrExclusionChecker, NodeExclusionChecker, NodeRemoveChecker, PluginPolicy,
    SanitizePluginDirective
};

use crate::Restrictive;

pub struct PluginPolicyBuilder<T: SanitizePluginDirective = Restrictive> {
    exclude_checkers: Vec<Box<dyn NodeExclusionChecker>>,
    remove_checkers: Vec<Box<dyn NodeRemoveChecker>>,
    attr_exclude_checkers: Vec<Box<dyn AttrExclusionChecker>>,
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

    pub fn exclude<C: NodeExclusionChecker + 'static>(mut self, checker: C) -> Self {
        self.exclude_checkers.push(Box::new(checker));
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