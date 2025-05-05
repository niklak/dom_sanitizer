use dom_query::NodeRef;
use dom_sanitizer::plugin_policy::preset;
use dom_sanitizer::plugin_policy::{NodeChecker, PluginPolicy};
use dom_sanitizer::Restrictive;

use html5ever::local_name;

struct ExcludeOnlyHttps;
impl NodeChecker for ExcludeOnlyHttps {
    fn is_match(&self, node: &NodeRef) -> bool {
        node.has_name("a")
            && node
                .attr("href")
                .map_or(false, |href| href.starts_with("https://"))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This example is using some predefined checkers from the `preset` module.

    // Creates a restrictive policy that allows only specific elements and attributes
    // which are explicitly excluded from sanitization with custom checkers.
    let policy: PluginPolicy<Restrictive> = PluginPolicy::builder()
        // Allow `a` elements only if their `href` starts with "https://"
        .exclude(ExcludeOnlyHttps)
        // Allow `title`, `p`, `mark`, and `b` elements
        .exclude(preset::MatchLocalNames(vec![
            local_name!("title"),
            local_name!("p"),
            local_name!("mark"),
            local_name!("b"),
        ]))
        // `html`, `head`, and `body` are always kept
        .build();

    let contents: &str = r#"
    <!DOCTYPE html>
    <html lang="en">
    <head><title>Test Ad Block</title></head>
        <body>
            <div><p role="paragraph">The first paragraph contains <a href="/first" role="link">the first link</a>.</p></div>
            <div><p role="paragraph">The second paragraph contains <a href="/second" role="link">the second link</a>.</p></div>
            <div><p role="paragraph">The third paragraph contains <a href="/third" role="link">the third link</a>.</p></div>
            <div><p id="highlight" role="paragraph"><mark>highlighted text</mark>, <b>bold text</b></p></div>
        </body>
    </html>"#;

    let doc = dom_query::Document::from(contents);

    policy.sanitize_document(&doc);

    // After sanitization:
    // - there are no `div` elements in the DOM
    assert!(!doc.select("div").exists());

    // All links are stripped, because it's not clear if they are secure. (Didn't match the policy)
    assert_eq!(doc.select("a").length(), 0);
    // `link` appears only as text inside `p` elements
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
    Ok(())
}
