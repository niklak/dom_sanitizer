use dom_sanitizer::plugin_policy::{AttrChecker, NodeChecker, PluginPolicy};
use dom_sanitizer::Permissive;

use dom_query::NodeRef;

use html5ever::{local_name, LocalName};

/// Matches nodes with a specific local name.
pub struct MatchLocalName(pub LocalName);
impl NodeChecker for MatchLocalName {
    fn is_match(&self, node: &NodeRef) -> bool {
        node.qual_name_ref()
            .is_some_and(|qual_name| self.0 == qual_name.local)
    }
}

struct SuspiciousAttr;
impl AttrChecker for SuspiciousAttr {
    fn is_match_attr(&self, _node: &NodeRef, attr: &html5ever::Attribute) -> bool {
        let attr_name = attr.name.local.as_ref().to_ascii_lowercase();
        attr_name != "onclick" && attr_name.starts_with("on")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Creates a permissive policy that allows all elements and attributes by default,
    // excluding those matched by custom checkers.
    let policy: PluginPolicy<Permissive> = PluginPolicy::builder()
        // `div` elements become disallowed and will be stripped from the DOM
        .exclude(MatchLocalName(local_name!("div")))
        // `style` elements will be completely removed from the DOM
        .remove(MatchLocalName(local_name!("style")))
        // Attributes that start with `on` and are not `onclick` will be removed
        .exclude_attr(SuspiciousAttr)
        .build();

    let contents: &str = r#"
    <!DOCTYPE html>
    <html lang="en">
    <head><title>Test Ad Block</title></head>
        <body>
            <style>@keyframes x{}</style>
            <div><p role="paragraph">The first paragraph contains <a href="/first" role="link">the first link</a>.</p></div>
            <div><p role="paragraph">The second paragraph contains <a href="/second" role="link">the second link</a>.</p></div>
            <div><p role="paragraph">The third paragraph contains <a href="/third" role="link">the third link</a>.</p></div>
            <div><p id="highlight" role="paragraph"><mark>highlighted text</mark>, <b>bold text</b></p></div>
            <div>
                <a style="animation-name:x" onanimationend="alert(1)"></a>
            </div>
        </body>
    </html>"#;

    let doc = dom_query::Document::from(contents);

    policy.sanitize_document(&doc);

    // The `style` element is removed from the DOM
    assert!(!doc.select("style").exists());
    // All `div` elements are removed from the DOM
    assert!(!doc.select("div").exists());
    // All 4 `<p>` elements remain
    assert_eq!(doc.select("p").length(), 4);
    // Suspicious attributes removed (e.g., `onanimationend`)
    assert!(!doc.select("a[onanimationend]").exists());
    Ok(())
}
