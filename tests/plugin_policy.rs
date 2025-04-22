use dom_query::{Document, NodeRef};
use html5ever::{local_name, LocalName};
use dom_sanitizer::plugin_policy::{
    preset, AttrChecker, NodeChecker, PluginPolicy
};
use dom_sanitizer::{Permissive, Restrictive};

mod data;
use data::PARAGRAPH_CONTENTS;

struct AllowOnlyHttps;
impl NodeChecker for AllowOnlyHttps {
    fn is_match(&self, node: &NodeRef) -> bool {
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
impl NodeChecker for AllowNonEmptyDiv {
    fn is_match(&self, node: &NodeRef) -> bool {
        if node.has_name("div") {
            return !node.text().is_empty();
        }
        false
    }
}

struct AllowP;
impl NodeChecker for AllowP {
    fn is_match(&self, node: &NodeRef) -> bool {
        node.has_name("p")
    }
}
struct ExcludeLocalName(LocalName);
impl NodeChecker for ExcludeLocalName {
    fn is_match(&self, node: &NodeRef) -> bool {
        let Some(qual_name) = node.qual_name_ref() else {
            return false;
        };
        qual_name.local == self.0
    }
}
struct MatchLocalNames(Vec<LocalName>);
impl NodeChecker for MatchLocalNames {
    fn is_match(&self, node: &NodeRef) -> bool {
        let Some(qual_name) = node.qual_name_ref() else {
            return false;
        };
        self.0.contains(&qual_name.local)
    }
}

#[test]
fn test_restrictive_plugin_policy() {

    let doc = Document::from(PARAGRAPH_CONTENTS);
    let policy: PluginPolicy<Restrictive> = PluginPolicy::builder()
        .exclude(AllowOnlyHttps)
        .exclude(AllowNonEmptyDiv)
        .exclude(AllowP)
        .exclude(preset::AllowBasicHtml)
        .exclude(ExcludeLocalName(local_name!("title")))
        .exclude(MatchLocalNames(vec![
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


#[test]
fn test_permissive_plugin_policy_remove() {
    let contents = include_str!("../test-pages/table.html");
    let doc = Document::from(contents);
    let policy: PluginPolicy<Permissive> = PluginPolicy::builder()
        .exclude(MatchLocalNames(vec![
            local_name!("style"),
        ]))
        .build();

    
    assert!(doc.select("style").exists());

    policy.sanitize_document(&doc);

    assert!(!doc.select("style").exists());

    // After sanitization, we got style inner contents without the `style` elements -- as a text node.
    assert!(doc.html().contains("border-collapse: collapse"));

    // For such cases it's better to use the `remove_elements()` method.
    let policy: PluginPolicy<Permissive> = PluginPolicy::builder()
    .remove(MatchLocalNames(vec![
        local_name!("style"),
    ]))
    .build();
    let doc = Document::from(contents);
    policy.sanitize_document(&doc);
    // in that case style elements are removed from the DOM tree, including their text content.
    assert!(!doc.select("style").exists());
    assert!(!doc.html().contains("border-collapse: collapse"));
}