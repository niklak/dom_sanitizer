#[derive(Debug, Clone, Copy, Default)]
pub enum Directive {
    Permit,
    #[default]
    Restrict,
}

#[derive(Debug, Clone, Default)]
pub struct AttributeRule<'a> {
    pub element: Option<&'a str>,
    pub attributes: Vec<&'a str>,
}

#[derive(Debug, Clone, Default)]
pub struct Policy<'a> {
    pub attr_rules: Vec<AttributeRule<'a>>,
    pub element_rules: Vec<&'a str>,
    pub directive: Directive,
}

impl Policy<'_> {
    pub fn sanitize_node(&self, node: &dom_query::NodeRef) {
        if self.element_rules.is_empty() && self.attr_rules.is_empty() {
            return;
        }

        let mut child = node.first_child();

        while let Some(ref child_node) = child {
            let next_node = child_node.next_sibling();
            if child_node.may_have_children() {
                self.sanitize_node(child_node);
            }
            if !child_node.is_element() {
                child = next_node;
                continue;
            }
            if child_node.qual_name_ref().map_or(false, |name| {
                self.element_rules.contains(&name.local.as_ref())
            }) {
                if let Some(first_inline) = child_node.first_child() {
                    child_node.insert_siblings_before(&first_inline);
                };
                child_node.remove_from_parent();
                continue;
            }
            self.sanitize_node_attr(child_node);
            child = next_node;
        }
    }
    fn sanitize_node_attr(&self, node: &dom_query::NodeRef) {
        if self.attr_rules.is_empty() {
            return;
        }

        for rule in &self.attr_rules {
            let Some(element_name) = rule.element else {
                node.remove_attrs(&rule.attributes);
                continue;
            };

            let Some(qual_name) = node.qual_name_ref() else {
                continue;
            };

            if qual_name.local.as_ref() == element_name {
                node.remove_attrs(&rule.attributes);
            }
        }
    }
}
