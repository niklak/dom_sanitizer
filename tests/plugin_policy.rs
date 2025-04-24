use dom_query::{Document, NodeRef};
use dom_sanitizer::plugin_policy::core::{PermissivePluginPolicy, RestrictivePluginPolicy};
use dom_sanitizer::plugin_policy::preset::SimpleMatchAttribute;
use dom_sanitizer::plugin_policy::{preset, AttrChecker, NodeChecker, PluginPolicy};
use dom_sanitizer::{Permissive, Restrictive};
use html5ever::{local_name, LocalName};

mod data;
use data::PARAGRAPH_CONTENTS;
use regex::Regex;

struct ExcludeOnlyHttps;
impl NodeChecker for ExcludeOnlyHttps {
    fn is_match(&self, node: &NodeRef) -> bool {
        node.has_name("a")
            && node
                .attr("href")
                .map_or(false, |href| href.starts_with("https://"))
    }
}

struct ExcludeNoHttps;
impl NodeChecker for ExcludeNoHttps {
    fn is_match(&self, node: &NodeRef) -> bool {
        if node.has_name("a") {
            let Some(href) = node.attr("href") else {
                return false;
            };
            return !href.starts_with("https://");
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

struct ExcludeP;
impl NodeChecker for ExcludeP {
    fn is_match(&self, node: &NodeRef) -> bool {
        node.has_name("p")
    }
}

struct SuspiciousAttr;
impl AttrChecker for SuspiciousAttr {
    fn is_match_attr(&self, _node: &NodeRef, attr: &html5ever::Attribute) -> bool {
        let attr_name = attr.name.local.as_ref();
        if attr_name != "onclick" && attr_name.starts_with("on") {
            return true;
        }
        false
    }
}

struct RegexContentCountMatcher {
    element_scope: LocalName,
    regex: Regex,
    threshold: usize,
}

impl RegexContentCountMatcher {
    fn new(re: &str, threshold: usize, element_scope: &str) -> Self {
        Self {
            element_scope: LocalName::from(element_scope),
            regex: Regex::new(re).unwrap(),
            threshold,
        }
    }
}

impl NodeChecker for RegexContentCountMatcher {
    fn is_match(&self, node: &NodeRef) -> bool {
        let Some(qual_name) = node.qual_name_ref() else {
            return false;
        };
        if qual_name.local != self.element_scope {
            return false;
        }
        let html = node.html();
        if html.is_empty() {
            return false;
        }

        self.regex.find_iter(&html).count() >= self.threshold
    }
}

#[test]
fn test_restrictive_plugin_policy() {
    let doc = Document::from(PARAGRAPH_CONTENTS);
    let policy: PluginPolicy<Restrictive> = PluginPolicy::builder()
        .exclude(ExcludeOnlyHttps)
        .exclude(AllowNonEmptyDiv)
        .exclude(ExcludeP)
        .exclude(preset::AllowBasicHtml)
        .exclude(preset::MatchLocalName(local_name!("title")))
        .exclude(preset::MatchLocalNames(vec![
            local_name!("mark"),
            local_name!("b"),
        ]))
        .build();

    policy.sanitize_node(&doc.root());
    // Divs are not empty, so they are allowed
    assert_eq!(doc.select("div").length(), 4);

    // All links are stripped, because it's not clear if they are secure.
    assert_eq!(doc.select("a").length(), 0);
    assert_eq!(doc.html().matches("link").count(), 3);

    // html, head, body are always kept
    assert!(doc.select("html").exists());
    assert!(doc.select("head").exists());
    assert!(doc.select("body").exists());
    // title is preserved, because it's excluded from the Restrictive policy
    assert!(doc.select("head title").exists());
    assert!(doc.select("p mark").exists());
    assert!(doc.select("p b").exists());
    assert!(!doc.select("p[role]").exists());
}

#[test]
fn test_restrictive_policy_attrs() {
    let policy: PluginPolicy<Restrictive> = PluginPolicy::builder()
        .exclude(preset::MatchLocalNames(vec![
            local_name!("p"),
            local_name!("a"),
        ]))
        .exclude_attr(SimpleMatchAttribute::new(None, local_name!("role")))
        .exclude_attr(SimpleMatchAttribute::new(None, local_name!("href")))
        .build();
    let doc = Document::from(PARAGRAPH_CONTENTS);
    policy.sanitize_document(&doc);
    assert!(!doc.select("div").exists());
    assert_eq!(doc.select("p > a[href]").length(), 3);
    assert_eq!(doc.select("[role]").length(), 7);
}

#[test]
fn test_restrictive_plugin_policy_remove() {
    let doc = Document::from(PARAGRAPH_CONTENTS);
    let policy: RestrictivePluginPolicy = PluginPolicy::builder()
        .remove(ExcludeNoHttps)
        .exclude(preset::AllowBasicHtml)
        .build();

    policy.sanitize_node(&doc.root());
    // Divs are not empty, so they are allowed
    assert_eq!(doc.select("div").length(), 0);

    // All links are stripped, because it's not clear if they are secure.
    assert_eq!(doc.select("a").length(), 0);
    assert_eq!(doc.html().matches("link").count(), 0);

    // html, head, body are always kept
    assert!(doc.select("html").exists());
    assert!(doc.select("head").exists());
    assert!(doc.select("body").exists());
}

#[test]
fn test_permissive_plugin_policy_remove() {
    let contents = include_str!("../test-pages/table.html");
    let doc = Document::from(contents);
    let policy: PluginPolicy<Permissive> = PluginPolicy::builder()
        .exclude(preset::MatchLocalNames(vec![local_name!("style")]))
        .build();

    assert!(doc.select("style").exists());

    policy.sanitize_document(&doc);

    assert!(!doc.select("style").exists());

    // After sanitization, we got style inner contents without the `style` elements -- as a text node.
    assert!(doc.html().contains("border-collapse: collapse"));

    // For such cases it's better to use the `remove_elements()` method.
    let policy: PluginPolicy<Permissive> = PluginPolicy::builder()
        .remove(preset::MatchLocalNames(vec![local_name!("style")]))
        .build();
    let doc = Document::from(contents);
    policy.sanitize_document(&doc);
    // in that case style elements are removed from the DOM tree, including their text content.
    assert!(!doc.select("style").exists());
    assert!(!doc.html().contains("border-collapse: collapse"));
}

#[test]
fn test_permissive_plugin_policy_unspecified() {
    let contents = include_str!("../test-pages/table.html");
    let doc = Document::from(contents);
    let policy: PermissivePluginPolicy = PluginPolicy::builder().build();

    let total_nodes_before = doc.root().descendants_it().count();
    policy.sanitize_document(&doc);
    let total_nodes_after = doc.root().descendants_it().count();
    assert_eq!(total_nodes_before, total_nodes_after);
}

#[test]
fn test_permissive_plugin_policy_exclude_attr() {
    let contents: &str = r#"
<!DOCTYPE html>
<html>
    <head><title>Test</title></head>
    <body>
        <style>@keyframes x{}</style>
        <a style="animation-name:x" onanimationend="alert(1)"></a>
        <p>Test content</p>
    </body>
</html>"#;

    let doc = Document::from(contents);
    let policy: PluginPolicy<Permissive> = PluginPolicy::builder()
        .exclude_attr(SuspiciousAttr)
        .remove(preset::MatchLocalNames(vec![local_name!("style")]))
        .build();

    assert!(doc.select("style").exists());
    assert!(doc.select("a[onanimationend]").exists());
    policy.sanitize_document(&doc);
    assert!(!doc.select("style").exists());
    assert!(!doc.select("a[onanimationend]").exists());
    assert!(doc.select("a").exists());
}

#[test]
fn test_permissive_plugin_policy_remove_by_regex() {
    let contents: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <title>Test Ad Block</title>
</head>
<body>

    <div class="ad-block">
        <h3 class="ad-title">Limited Time Offer!</h3>
        <p class="ad-text">Discover amazing deals on our latest products. Shop now and save big!</p>
        <a href="/deal" target="_blank">Learn More</a>
    </div>

    <div>
        <p class="regular-text">A test paragraph.</p>
    </div>

    <div>
        <p>Another test paragraph.</p>
    </div>

</body>
</html>"#;

    let doc = Document::from(contents);
    let policy: PluginPolicy<Permissive> = PluginPolicy::builder()
        .remove(RegexContentCountMatcher::new(
            r"(?i)shop now|amazing deals|offer",
            3,
            "div",
        ))
        .build();
    assert!(doc.select("div.ad-block").exists());
    assert_eq!(doc.select("div").length(), 3);
    assert_eq!(doc.select("p").length(), 3);

    policy.sanitize_document(&doc);
    assert!(!doc.select("div.ad-block").exists());
    assert_eq!(doc.select("div").length(), 2);
    assert_eq!(doc.select("p").length(), 2);
}
