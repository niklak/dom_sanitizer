use dom_query::NodeRef;
use html5ever::local_name;

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
