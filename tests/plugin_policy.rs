use dom_query::{Document, NodeRef};
use html5ever::{local_name, LocalName};
use dom_sanitizer::plugin_policy::{
    AttrExclusionChecker, NodeExclusionChecker, NodeRemoveChecker, PluginPolicy,
};
use dom_sanitizer::Restrictive;

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