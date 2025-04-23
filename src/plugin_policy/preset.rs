use dom_query::NodeRef;
use html5ever::{local_name, Attribute, LocalName};

use super::{core::NodeChecker, AttrChecker};

/// Matches basic HTML structure elements: html, head, and body.
pub struct AllowBasicHtml;
impl NodeChecker for AllowBasicHtml {
    fn is_match(&self, node: &NodeRef) -> bool {
        node.qual_name_ref().map_or(false, |qual_name| {
            matches!(
                qual_name.local,
                local_name!("html") | local_name!("head") | local_name!("body")
            )
        })
    }
}

/// Matches nodes with a specific local name.
pub struct MatchLocalName(pub LocalName);
impl NodeChecker for MatchLocalName {
    fn is_match(&self, node: &NodeRef) -> bool {
        node.qual_name_ref().map_or(false, |qual_name| self.0 == qual_name.local)

    }
}
/// Matches nodes with local names contained in the provided vector.
pub struct MatchLocalNames(pub Vec<LocalName>);
impl NodeChecker for MatchLocalNames {
    fn is_match(&self, node: &NodeRef) -> bool {
        node.qual_name_ref().map_or(false, |qual_name| self.0.contains(&qual_name.local))
    }
}

/// Matches nodes with a specific local name and checks if the attribute matches.
pub struct SimpleMatchAttribute{
    /// The local name of the element to match. If `None`, matches any element.
    pub element_scope: Option<LocalName>,
    /// The local name of the attribute to match.
    pub attribute_name: LocalName,
}

impl AttrChecker for SimpleMatchAttribute {
    fn is_match_attr(&self, node: &NodeRef, attr: &Attribute) -> bool {
        let Some(ref element_scope) = self.element_scope else {
            return attr.name.local == self.attribute_name;
        };

        if node.qual_name_ref().map_or(false,|name| &name.local != element_scope) {
            return false;
        }
        attr.name.local == self.attribute_name
    }
}

impl SimpleMatchAttribute {
    /// Creates a new `SimpleMatchAttribute` instance.
    ///
    /// # Arguments
    ///
    /// * `element_scope` - The local name of the element to match. If `None`, matches any element.
    /// * `attribute_name` - The local name of the attribute to match.
    pub fn new(element_scope: Option<LocalName>, attribute_name: LocalName) -> Self {
        Self {
            element_scope,
            attribute_name,
        }
    }
}