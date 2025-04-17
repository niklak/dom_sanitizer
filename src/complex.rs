use dom_query::NodeRef;

pub trait NodeAllowChecker {
    /// Checks if the node is allowed by the policy.
    fn is_allowed(&self, _node: &NodeRef) -> bool {
        // Default implementation allows all nodes.
        false
    }
}

pub struct RestrictivePluginPolicy {
    allow_checkers: Vec<Box<dyn NodeAllowChecker>>,
}

impl RestrictivePluginPolicy {
    fn is_node_allowed(&self, node: &NodeRef) -> bool {
        for checker in &self.allow_checkers {
            if checker.is_allowed(node) {
                return true;
            }
        }
        false
    }

    pub fn sanitize_node(&self, node: &NodeRef) {
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
            if self.is_node_allowed(child_node) {
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
    checkers: Vec<Box<dyn NodeAllowChecker>>,
}

impl RestrictiveComplexPolicyBuilder {
    pub fn new() -> Self {
        RestrictiveComplexPolicyBuilder { checkers: Vec::new() }
    }

    pub fn allow<T: NodeAllowChecker + 'static>(mut self, checker: T) -> Self {
        self.checkers.push(Box::new(checker));
        self
    }

    pub fn build(self) -> RestrictivePluginPolicy {
        RestrictivePluginPolicy {
            allow_checkers: self.checkers,
        }
    }
}


mod tests {
    use super::*;
    use dom_query::{Document, NodeRef};
    use html5ever::{local_name, LocalName};

    struct AllowOnlyHttps;
    impl NodeAllowChecker for AllowOnlyHttps {
        fn is_allowed(&self, node: &NodeRef) -> bool {
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
        fn is_allowed(&self, node: &NodeRef) -> bool {
            if node.has_name("div") {
                return !node.text().is_empty();
            }
            false
        }
    }

    struct AllowP;
    impl NodeAllowChecker for AllowP {
        fn is_allowed(&self, node: &NodeRef) -> bool {
            if node.has_name("p") {
                return true;
            }
            false
        }
    }

    struct AllowBaseHtml;
    impl NodeAllowChecker for AllowBaseHtml {
        fn is_allowed(&self, node: &NodeRef) -> bool {
            let Some(qual_name) = node.qual_name_ref() else {
                return false;
            };
            matches!(qual_name.local, local_name!("html") | local_name!("head") | local_name!("body"))
        }
    }
    struct AllowCustomLocalName(LocalName);
    impl NodeAllowChecker for AllowCustomLocalName {
        fn is_allowed(&self, node: &NodeRef) -> bool {
            let Some(qual_name) = node.qual_name_ref() else {
                return false;
            };
            qual_name.local == self.0 
        }
    }
    struct AllowCustomLocalNames(Vec<LocalName>);
    impl NodeAllowChecker for AllowCustomLocalNames {
        fn is_allowed(&self, node: &NodeRef) -> bool {
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
        let policy= RestrictivePluginPolicy::builder()
            .allow(AllowOnlyHttps)
            .allow(AllowNonEmptyDiv)
            .allow(AllowP)
            .allow(AllowBaseHtml)
            .allow(AllowCustomLocalName(local_name!("title")))
            .allow(AllowCustomLocalNames(vec![local_name!("mark"), local_name!("b")]))
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
