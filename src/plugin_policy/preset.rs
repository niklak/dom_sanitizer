use dom_query::NodeRef;
use html5ever::{Attribute, LocalName, Namespace};

use super::{core::NodeChecker, AttrChecker};

/// Matches nodes with a specific local name.
pub struct LocalNameMatcher(pub LocalName);
impl NodeChecker for LocalNameMatcher {
    fn is_match(&self, node: &NodeRef) -> bool {
        node.qual_name_ref()
            .is_some_and(|qual_name| self.0 == qual_name.local)
    }
}

impl LocalNameMatcher {
    /// Creates a new `MatchLocalName` instance.
    ///
    /// # Arguments
    ///
    /// * `name` - The local name to match.
    pub fn new(name: &str) -> Self {
        Self(LocalName::from(name))
    }
}

/// Matches nodes with local names contained in the provided vector.
pub struct LocalNamesMatcher(pub Vec<LocalName>);
impl NodeChecker for LocalNamesMatcher {
    fn is_match(&self, node: &NodeRef) -> bool {
        node.qual_name_ref()
            .is_some_and(|qual_name| self.0.contains(&qual_name.local))
    }
}

impl LocalNamesMatcher {
    /// Creates a new `MatchLocalNames` instance.
    ///
    /// # Arguments
    ///
    /// * `names` - A vector of local names to match.
    pub fn new(names: &[&str]) -> Self {
        Self(names.iter().map(|name| LocalName::from(*name)).collect())
    }
}

/// Matches nodes with a specific local name and checks if the attribute matches.
pub struct AttrMatcher {
    /// The local name of the element to match. If `None`, matches any element.
    pub element_scope: Option<LocalName>,
    /// The local names of the attributes to match.
    pub attr_names: Vec<LocalName>,
}

impl AttrChecker for AttrMatcher {
    fn is_match_attr(&self, node: &NodeRef, attr: &Attribute) -> bool {
        let Some(ref element_scope) = self.element_scope else {
            return self.attr_names.contains(&attr.name.local);
        };
        // Only proceed if node's local name matches the element scope
        if !node
            .qual_name_ref()
            .is_some_and(|name| &name.local == element_scope)
        {
            return false;
        }
        self.attr_names.contains(&attr.name.local)
    }
}

impl AttrMatcher {
    /// Creates a new `AttrMatcher` instance.
    ///
    /// # Arguments
    ///
    /// * `element_scope` - The name of the element to match. If `None`, matches any element.
    /// * `attr_names` - The local name of the attribute to match.
    pub fn new(element_scope: Option<&str>, attr_names: &[&str]) -> Self {
        Self {
            element_scope: element_scope.map(LocalName::from),
            attr_names: attr_names
                .iter()
                .map(|name| LocalName::from(*name))
                .collect(),
        }
    }
}

/// Matches nodes with a specific namespace and checks if the attribute matches.
pub struct NsAttrMatcher {
    /// The namespace of the element to match.
    pub ns: Namespace,
    /// The local names of the attributes to match.
    pub attr_names: Vec<LocalName>,
}

impl AttrChecker for NsAttrMatcher {
    fn is_match_attr(&self, node: &NodeRef, attr: &Attribute) -> bool {
        // Only proceed if node's namespace matches the element scope
        if !node.qual_name_ref().is_some_and(|name| name.ns == self.ns) {
            return false;
        }
        self.attr_names.contains(&attr.name.local)
    }
}

impl NsAttrMatcher {
    /// Creates a new `AttrMatcher` instance.
    ///
    /// # Arguments
    ///
    /// * `ns` - The namespace of the element to match.
    /// * `attr_names` - The local name of the attribute to match.
    pub fn new(ns: &str, attr_names: &[&str]) -> Self {
        Self {
            ns: Namespace::from(ns),
            attr_names: attr_names
                .iter()
                .map(|name| LocalName::from(*name))
                .collect(),
        }
    }
}

/// A matcher that checks if a node's namespace matches the specified namespace.
pub struct NamespaceMatcher(pub Namespace);

impl NamespaceMatcher {
    /// Creates a new `NamespaceMatcher` instance.
    ///
    /// # Arguments
    ///
    /// * `ns` - The namespace to match.
    ///
    /// # Examples
    ///
    /// let svg_matcher = NamespaceMatcher::new("http://www.w3.org/2000/svg");
    pub fn new(namespace: &str) -> Self {
        Self(Namespace::from(namespace))
    }
}

impl NodeChecker for NamespaceMatcher {
    fn is_match(&self, node: &NodeRef) -> bool {
        node.qual_name_ref().is_some_and(|name| name.ns == self.0)
    }
}
