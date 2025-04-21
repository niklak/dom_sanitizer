use dom_query::NodeRef;
use html5ever::Attribute;

use crate::{Permissive, Restrictive};
pub trait NodeExclusionChecker {
    /// Checks if the node is allowed by the policy.
    fn should_exclude(&self, _node: &NodeRef) -> bool {
        // Default implementation allows all nodes.
        false
    }
}

pub trait AttrExclusionChecker {
    fn should_exclude_attr(&self, _node: &NodeRef, _attr: &Attribute) -> bool {
        false
    }
}

pub trait NodeRemoveChecker {
    fn should_remove(&self, _node: &NodeRef) -> bool {
        false
    }
}

/// A trait for sanitization directives, defines methods for node and attribute sanitization.
pub trait SanitizePluginDirective {
    /// Sanitizes a node by removing elements and attributes based on the policy.
    fn sanitize_node(policy: &PluginPolicy<Self>, node: &NodeRef)
    where
        Self: Sized;
    /// Sanitizes the attributes of a node by removing or retaining them based on the policy.
    fn sanitize_node_attrs(policy: &PluginPolicy<Self>, node: &dom_query::NodeRef)
    where
        Self: Sized;
}

impl SanitizePluginDirective for Permissive {
    fn sanitize_node(policy: &PluginPolicy<Self>, node: &NodeRef) {
        if policy.exclude_checkers.is_empty()
            && policy.remove_checkers.is_empty()
            && policy.attr_exclude_checkers.is_empty()
        {
            return;
        }

        let mut child = node.first_child();

        while let Some(ref child_node) = child {
            let next_node = child_node.next_sibling();
            if policy.should_remove(child_node) {
                child_node.remove_from_parent();
                child = next_node;
                continue;
            }
            if child_node.may_have_children() {
                Self::sanitize_node(policy, child_node);
            }

            if policy.should_exclude(child_node) {
                if let Some(first_inline) = child_node.first_child() {
                    child_node.insert_siblings_before(&first_inline);
                };
                child_node.remove_from_parent();
            }
            Self::sanitize_node_attrs(policy, child_node);
            child = next_node;
        }
    }

    fn sanitize_node_attrs(policy: &PluginPolicy<Self>, node: &dom_query::NodeRef) {
        if policy.attr_exclude_checkers.is_empty() {
            return;
        }

        policy.exclude_attrs(node, |node, attrs| node.remove_attrs(attrs));
    }
}

impl SanitizePluginDirective for Restrictive {
    fn sanitize_node(policy: &PluginPolicy<Self>, node: &NodeRef) {
        let mut child = node.first_child();

        while let Some(ref child_node) = child {
            let next_node = child_node.next_sibling();

            if policy.should_remove(child_node) {
                child_node.remove_from_parent();
                child = next_node;
                continue;
            }
            if child_node.may_have_children() {
                policy.sanitize_node(child_node);
            }
            if !child_node.is_element() {
                child = next_node;
                continue;
            }
            if policy.should_exclude(child_node) {
                Self::sanitize_node_attrs(policy, child_node);
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

    fn sanitize_node_attrs(policy: &PluginPolicy<Self>, node: &dom_query::NodeRef) {
        if policy.attr_exclude_checkers.is_empty() {
            node.remove_all_attrs();
            return;
        }

        policy.exclude_attrs(node, |node, attrs| node.retain_attrs(attrs));
    }
}

pub struct PluginPolicy<T: SanitizePluginDirective = Restrictive> {
    exclude_checkers: Vec<Box<dyn NodeExclusionChecker>>,
    remove_checkers: Vec<Box<dyn NodeRemoveChecker>>,
    attr_exclude_checkers: Vec<Box<dyn AttrExclusionChecker>>,
    pub(crate) _directive: std::marker::PhantomData<T>,
}

impl<T: SanitizePluginDirective> PluginPolicy<T> {
    fn should_exclude(&self, node: &NodeRef) -> bool {
        for checker in &self.exclude_checkers {
            if checker.should_exclude(node) {
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
    fn should_exclude_attr(&self, node: &NodeRef, attr: &Attribute) -> bool {
        for checker in &self.attr_exclude_checkers {
            if checker.should_exclude_attr(node, attr) {
                return true;
            }
        }
        false
    }

    fn exclude_attrs<F>(&self, node: &NodeRef, exclude_fn: F)
    where
        F: FnOnce(&NodeRef, &[&str]),
    {
        let node_attrs = node.attrs();
        let attrs: Vec<&str> = node_attrs
            .iter()
            .filter(|a| self.should_exclude_attr(node, a))
            .map(|a| a.name.local.as_ref())
            .collect();
        exclude_fn(node, &attrs)
    }

    pub fn sanitize_node(&self, node: &NodeRef) {
        T::sanitize_node(self, node);
        node.normalize();
    }
}

impl<T: SanitizePluginDirective> PluginPolicy<T> {
    pub fn builder() -> PluginPolicyBuilder<T> {
        PluginPolicyBuilder::new()
    }
}

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

mod tests {
    use super::*;
    use dom_query::{Document, NodeRef};
    use html5ever::{local_name, LocalName};

    struct AllowOnlyHttps;
    impl NodeExclusionChecker for AllowOnlyHttps {
        fn should_exclude(&self, node: &NodeRef) -> bool {
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
    impl NodeExclusionChecker for AllowNonEmptyDiv {
        fn should_exclude(&self, node: &NodeRef) -> bool {
            if node.has_name("div") {
                return !node.text().is_empty();
            }
            false
        }
    }

    struct AllowP;
    impl NodeExclusionChecker for AllowP {
        fn should_exclude(&self, node: &NodeRef) -> bool {
            node.has_name("p")
        }
    }

    struct AllowBaseHtml;
    impl NodeExclusionChecker for AllowBaseHtml {
        fn should_exclude(&self, node: &NodeRef) -> bool {
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
    impl NodeExclusionChecker for AllowCustomLocalName {
        fn should_exclude(&self, node: &NodeRef) -> bool {
            let Some(qual_name) = node.qual_name_ref() else {
                return false;
            };
            qual_name.local == self.0
        }
    }
    struct AllowCustomLocalNames(Vec<LocalName>);
    impl NodeExclusionChecker for AllowCustomLocalNames {
        fn should_exclude(&self, node: &NodeRef) -> bool {
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
        let policy: PluginPolicy<Restrictive> = PluginPolicy::builder()
            .exclude(AllowOnlyHttps)
            .exclude(AllowNonEmptyDiv)
            .exclude(AllowP)
            .exclude(AllowBaseHtml)
            .exclude(AllowCustomLocalName(local_name!("title")))
            .exclude(AllowCustomLocalNames(vec![
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
