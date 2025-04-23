use dom_query::NodeRef;
use html5ever::{local_name, LocalName};

use super::core::NodeChecker;

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


pub struct MatchLocalName(pub LocalName);
impl NodeChecker for MatchLocalName {
    fn is_match(&self, node: &NodeRef) -> bool {
        node.qual_name_ref().map_or(false, |qual_name| self.0 == qual_name.local)

    }
}
pub struct MatchLocalNames(pub Vec<LocalName>);
impl NodeChecker for MatchLocalNames {
    fn is_match(&self, node: &NodeRef) -> bool {
        node.qual_name_ref().map_or(false, |qual_name| self.0.contains(&qual_name.local))
    }
}