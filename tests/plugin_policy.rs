use dom_query::{Document, NodeRef};
use dom_sanitizer::plugin_policy::core::{PermissivePluginPolicy, RestrictivePluginPolicy};
use dom_sanitizer::plugin_policy::preset::AttrMatcher;
use dom_sanitizer::plugin_policy::{preset, AttrChecker, NodeChecker, PluginPolicy};
use dom_sanitizer::{Permissive, Restrictive};
use html5ever::{ns, LocalName};

mod data;
use data::{PARAGRAPH_CONTENTS, SVG_CONTENTS};
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
        node.has_name("a")
            && node
                .attr("href")
                .map_or(false, |href| !href.starts_with("https://"))
    }
}

struct ExcludeNonEmptyDiv;
impl NodeChecker for ExcludeNonEmptyDiv {
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
        let attr_name = attr.name.local.as_ref().to_ascii_lowercase();
        attr_name != "onclick" && attr_name.starts_with("on")
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

        let text = node.text();
        if text.is_empty() {
            return false;
        }

        self.regex.find_iter(&text).count() >= self.threshold
    }
}

#[test]
fn test_restrictive_plugin_policy() {
    let doc = Document::from(PARAGRAPH_CONTENTS);
    // The policy is restrictive, so it will remove all elements that are not explicitly allowed (excluded from the policy).
    let policy: PluginPolicy<Restrictive> = PluginPolicy::builder()
        .exclude(ExcludeOnlyHttps)
        .exclude(ExcludeNonEmptyDiv)
        .exclude(ExcludeP)
        .exclude(preset::LocalNameMatcher::new("title"))
        .exclude(preset::LocalNamesMatcher::new(&["mark", "b"]))
        .build();

    policy.sanitize_node(&doc.root());
    // Divs are not empty, so they are allowed
    assert_eq!(doc.select("div").length(), 4);

    // All links are stripped, because it's not clear if they are secure. (Didn't match the policy)
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
        .exclude(preset::LocalNamesMatcher::new(&["p", "a"]))
        .exclude_attr(AttrMatcher::new(None, &["role"]))
        .exclude_attr(AttrMatcher::new(Some("a"), &["href"]))
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
    let policy: RestrictivePluginPolicy = PluginPolicy::builder().remove(ExcludeNoHttps).build();

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
        .exclude(preset::LocalNamesMatcher::new(&["style"]))
        .build();

    assert!(doc.select("style").exists());

    policy.sanitize_document(&doc);

    assert!(!doc.select("style").exists());

    // After sanitization, we got style inner contents without the `style` elements -- as a text node.
    assert!(doc.html().contains("border-collapse: collapse"));

    // For such cases it's better to use the `remove_elements()` method.
    let policy: PluginPolicy<Permissive> = PluginPolicy::builder()
        .remove(preset::LocalNamesMatcher::new(&["style"]))
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
        .remove(preset::LocalNamesMatcher::new(&["style"]))
        .build();

    assert!(doc.select("style").exists());
    assert!(doc.select("a[onanimationend]").exists());
    policy.sanitize_document(&doc);
    assert!(!doc.select("style").exists());
    assert!(!doc.select("a[onanimationend]").exists());
    assert!(doc.select("a").exists());

    assert!(contents.contains(r#"onanimationend="alert(1)""#));
    let html = policy.sanitize_html(contents);
    assert!(!html.contains(r#"onanimationend="alert(1)""#));
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

#[test]
fn test_plugin_policy_debug_fmt() {
    let policy: PluginPolicy<Restrictive> = PluginPolicy::builder()
        .exclude(crate::preset::LocalNameMatcher::new("div"))
        .remove(crate::preset::LocalNameMatcher::new("style"))
        .exclude_attr(crate::preset::AttrMatcher::new(None, &["role"]))
        .build();

    let debug_output = format!("{:?}", policy);

    assert!(debug_output.contains("PluginPolicy"));
    assert!(debug_output.contains("exclude_checkers: Arc<[Box<dyn NodeChecker>]> (1 elements)"));
    assert!(debug_output.contains("remove_checkers: Arc<[Box<dyn NodeChecker>]> (1 elements)"));
    assert!(
        debug_output.contains("attr_exclude_checkers: Arc<[Box<dyn AttrChecker>]> (1 elements)")
    );
    assert!(
        debug_output.contains("_directive: PhantomData<dom_sanitizer::directives::Restrictive>")
    );
}

#[test]
fn test_permissive_plugin_policy_svg() {
    let policy = PermissivePluginPolicy::builder()
        .exclude(preset::NamespaceMatcher::new("http://www.w3.org/2000/svg"))
        .exclude(preset::LocalNameMatcher::new("div"))
        .build();

    let doc = Document::from(SVG_CONTENTS);

    assert!(doc.select("svg").exists());
    assert!(doc.select("rect").exists());
    assert!(doc.select("div").exists());

    policy.sanitize_document(&doc);

    // The policy should remove the SVG element and its contents
    assert!(!doc.select("svg").exists());
    assert!(!doc.select("rect").exists());
    assert!(!doc.select("div").exists());
}

#[test]
fn test_permissive_policy_svg_class() {
    let policy = PermissivePluginPolicy::builder()
        .exclude_attr(preset::NsAttrMatcher::new(
            "http://www.w3.org/2000/svg",
            &["class", "style"],
        ))
        .build();

    let doc = Document::from(SVG_CONTENTS);
    assert!(doc.select("svg *[style]").exists());
    assert!(doc.select("svg *[class]").exists());

    policy.sanitize_document(&doc);

    assert!(!doc.select("svg *[style]").exists());
    assert!(!doc.select("svg *[class]").exists());
    assert!(doc.select("div[class]").exists());
}

#[test]
fn test_restrictive_plugin_policy_svg() {
    struct SvgSafeAttrs;

    impl AttrChecker for SvgSafeAttrs {
        fn is_match_attr(&self, node: &NodeRef, attr: &html5ever::Attribute) -> bool {
            if !node
                .qual_name_ref()
                .map_or(false, |name| name.ns == ns!(svg))
            {
                return false;
            }
            !attr.name.local.to_ascii_lowercase().starts_with("on")
        }
    }

    let policy = RestrictivePluginPolicy::builder()
        .exclude(preset::NamespaceMatcher::new("http://www.w3.org/2000/svg"))
        .exclude(preset::LocalNameMatcher::new("div"))
        .exclude_attr(SvgSafeAttrs)
        .build();

    let doc = Document::from(SVG_CONTENTS);

    assert!(doc
        .select("svg[style][oncontentvisibilityautostatechange]")
        .exists());
    assert!(doc.select("rect[width][height][style]").exists());
    assert!(doc.select("div").exists());
    assert!(doc.select("p").exists());

    policy.sanitize_document(&doc);

    assert!(!doc
        .select("svg[oncontentvisibilityautostatechange]")
        .exists());
    assert!(doc.select("svg[style]").exists());
    assert!(doc.select("rect[width][height][style]").exists());
    assert!(doc.select("div").exists());
    assert!(!doc.select("p").exists());
}
