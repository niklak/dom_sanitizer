use dom_query::NodeRef;
use html5ever::local_name;

use super::core::NodeChecker;

pub struct AllowBasicHtml;
impl NodeChecker for AllowBasicHtml {
    fn is_match(&self, node: &NodeRef) -> bool {
        let Some(qual_name) = node.qual_name_ref() else {
            return false;
        };
        matches!(
            qual_name.local,
            local_name!("html") | local_name!("head") | local_name!("body")
        )
    }
}