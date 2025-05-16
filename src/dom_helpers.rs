use dom_query::NodeRef;

pub(crate) fn next_child_or_sibling<'a>(
    node: &NodeRef<'a>,
    ignore_child: bool,
    scope: &NodeRef<'a>,
) -> Option<NodeRef<'a>> {
    if !ignore_child {
        if let Some(first_child) = node.first_element_child() {
            return Some(first_child);
        }
    }

    if let Some(sibling) = node.next_element_sibling() {
        return Some(sibling);
    }
    let mut parent = node.parent();
    while let Some(parent_node) = parent {
        if parent_node.id == scope.id {
            return None;
        }
        if let Some(next_sibling) = parent_node.next_element_sibling() {
            return Some(next_sibling);
        } else {
            parent = parent_node.parent()
        }
    }
    None
}
