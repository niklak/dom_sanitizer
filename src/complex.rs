use dom_query::NodeRef;
use html5ever::{Attribute, LocalName};

pub trait NodeAllowChecker {
    /// Checks if the node is allowed by the policy.
    fn should_allow(&self, _node: &NodeRef) -> bool {
        // Default implementation allows all nodes.
        false
    }
}

pub trait AttributeAllowChecker {
    fn should_allow_attribute(&self, _node: &NodeRef, _attr: &Attribute) -> bool {
        false
    }
}

pub trait NodeRemoveChecker {
    fn should_remove(&self, _node: &NodeRef) -> bool {
        false
    }
}

pub struct RestrictivePluginPolicy {
    allow_checkers: Vec<Box<dyn NodeAllowChecker>>,
    remove_checkers: Vec<Box<dyn NodeRemoveChecker>>,
    attribute_allow_checkers: Vec<Box<dyn AttributeAllowChecker>>,
}

impl RestrictivePluginPolicy {
    fn should_allow(&self, node: &NodeRef) -> bool {
        for checker in &self.allow_checkers {
            if checker.should_allow(node) {
                return true;
            }
        }
        false
    }

    fn should_remove(&self, node: &NodeRef) -> bool {
        for checker in &self.remove_checkers {
            if checker.should_remove(node) {
                return true;
            }
        }
        false
    }
    fn should_allow_attribute(&self, node: &NodeRef, attr: &Attribute) -> bool {
        for checker in &self.attribute_allow_checkers {
            if checker.should_allow_attribute(node, attr) {
                return true;
            }
        }
        false
    }

    fn allowed_attrs(&self, node: &NodeRef) -> Vec<LocalName> {
        let mut attrs: Vec<LocalName> = vec![];
        for attr in node.attrs().iter() {
            if self.should_allow_attribute(node, attr) {
                attrs.push(attr.name.local.clone());
            }
        }
        attrs
    }

    fn sanitize_node_attrs(&self, node: &NodeRef) {
        if self.attribute_allow_checkers.is_empty() {
            node.remove_all_attrs();
            return;
        }

        let allowed_attrs = self.allowed_attrs(node);
        let allowed_attrs_str = allowed_attrs
            .iter()
            .map(|name| name.as_ref())
            .collect::<Vec<_>>();
        node.retain_attrs(&allowed_attrs_str);
    }

    pub fn sanitize_node(&self, node: &NodeRef) {
        let mut child = node.first_child();

        while let Some(ref child_node) = child {
            let next_node = child_node.next_sibling();

            if self.should_remove(child_node) {
                child_node.remove_from_parent();
                child = next_node;
                continue;
            }
            if child_node.may_have_children() {
                self.sanitize_node(child_node);
            }
            if !child_node.is_element() {
                child = next_node;
                continue;
            }
            if self.should_allow(child_node) {
                self.sanitize_node_attrs(child_node);
                child = next_node;
                continue;
            }

            if let Some(first_inline) = child_node.first_child() {
                child_node.insert_siblings_before(&first_inline);
            };
            child_node.remove_from_parent();
            child = next_node;
        }
    }
}

impl RestrictivePluginPolicy {
    pub fn builder() -> RestrictiveComplexPolicyBuilder {
        RestrictiveComplexPolicyBuilder::new()
    }
}

pub struct RestrictiveComplexPolicyBuilder {
    allow_checkers: Vec<Box<dyn NodeAllowChecker>>,
    remove_checkers: Vec<Box<dyn NodeRemoveChecker>>,
    attribute_allow_checkers: Vec<Box<dyn AttributeAllowChecker>>,
}

impl RestrictiveComplexPolicyBuilder {
    pub fn new() -> Self {
        RestrictiveComplexPolicyBuilder {
            allow_checkers: vec![],
            remove_checkers: vec![],
            attribute_allow_checkers: vec![],
        }
    }

    pub fn allow<T: NodeAllowChecker + 'static>(mut self, checker: T) -> Self {
        self.allow_checkers.push(Box::new(checker));
        self
    }

    pub fn build(self) -> RestrictivePluginPolicy {
        RestrictivePluginPolicy {
            allow_checkers: self.allow_checkers,
            remove_checkers: self.remove_checkers,
            attribute_allow_checkers: self.attribute_allow_checkers,
        }
    }
}

mod tests {
    use super::*;
    use dom_query::{Document, NodeRef};
    use html5ever::{local_name, LocalName};

    struct AllowOnlyHttps;
    impl NodeAllowChecker for AllowOnlyHttps {
        fn should_allow(&self, node: &NodeRef) -> bool {
            if node.has_name("a") {
                let Some(href) = node.attr("href") else {
                    return false;
                };
                return href.starts_with("https://");
            }
            false
        }
    }

    struct AllowNonEmptyDiv;
    impl NodeAllowChecker for AllowNonEmptyDiv {
        fn should_allow(&self, node: &NodeRef) -> bool {
            if node.has_name("div") {
                return !node.text().is_empty();
            }
            false
        }
    }

    struct AllowP;
    impl NodeAllowChecker for AllowP {
        fn should_allow(&self, node: &NodeRef) -> bool {
            node.has_name("p")
        }
    }

    struct AllowBaseHtml;
    impl NodeAllowChecker for AllowBaseHtml {
        fn should_allow(&self, node: &NodeRef) -> bool {
            let Some(qual_name) = node.qual_name_ref() else {
                return false;
            };
            matches!(
                qual_name.local,
                local_name!("html") | local_name!("head") | local_name!("body")
            )
        }
    }
    struct AllowCustomLocalName(LocalName);
    impl NodeAllowChecker for AllowCustomLocalName {
        fn should_allow(&self, node: &NodeRef) -> bool {
            let Some(qual_name) = node.qual_name_ref() else {
                return false;
            };
            qual_name.local == self.0
        }
    }
    struct AllowCustomLocalNames(Vec<LocalName>);
    impl NodeAllowChecker for AllowCustomLocalNames {
        fn should_allow(&self, node: &NodeRef) -> bool {
            let Some(qual_name) = node.qual_name_ref() else {
                return false;
            };
            self.0.contains(&qual_name.local)
        }
    }

    #[test]
    fn test_restrictive_complex_policy() {
        let contents: &str = r#"
        <!DOCTYPE html>
        <html>
            <head><title>Test</title></head>
            <body>
                <div><p role="paragraph">The first paragraph contains <a href="/first" role="link">the first link</a>.</p></div>
                <div><p role="paragraph">The second paragraph contains <a href="/second" role="link">the second link</a>.</p></div>
                <div><p role="paragraph">The third paragraph contains <a href="/third" role="link">the third link</a>.</p></div>
                <div><p role="paragraph"><mark>highlighted text</mark>, <b>bold text</b></p></div>
                <div></div>
            </body>
        </html>"#;
        let doc = Document::from(contents);
        let policy = RestrictivePluginPolicy::builder()
            .allow(AllowOnlyHttps)
            .allow(AllowNonEmptyDiv)
            .allow(AllowP)
            .allow(AllowBaseHtml)
            .allow(AllowCustomLocalName(local_name!("title")))
            .allow(AllowCustomLocalNames(vec![
                local_name!("mark"),
                local_name!("b"),
            ]))
            .build();

        policy.sanitize_node(&doc.root());
        // Divs are not empty, so they are allowed
        assert_eq!(doc.select("div").length(), 4);

        // All links are stripped, because it's not clear if they are secure.
        assert_eq!(doc.select("a").length(), 0);

        // html, head, body are always kept
        assert!(doc.select("html").exists());
        assert!(doc.select("head").exists());
        assert!(doc.select("body").exists());
        // title is stripped, because it's not allowed by the policy
        assert!(doc.select("head title").exists());
        assert!(doc.select("p mark").exists());
        assert!(doc.select("p b").exists());
    }
}
