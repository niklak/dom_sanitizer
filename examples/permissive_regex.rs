use dom_sanitizer::plugin_policy::{NodeChecker, PluginPolicy};
use dom_sanitizer::Permissive;

use dom_query::NodeRef;

use html5ever::LocalName;
use regex::Regex;

// `RegexContentCountMatcher` checks whether a given regex pattern appears
// in the text content of a node a certain number of times. If the number
// of matches is greater than or equal to the specified threshold, the node
// is considered a match.
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let policy: PluginPolicy<Permissive> = PluginPolicy::builder()
        .remove(RegexContentCountMatcher::new(
            r"(?i)shop now|amazing deals|offer",
            3,
            "div",
        ))
        .build();

    let contents: &str = r#"
    <html lang="en">
        <head><title>Test Ad Block</title></head>
        <body>
            <div class="ad-block">
                <h3 class="ad-title">Limited Time Offer!</h3>
                <p class="ad-text">Discover amazing deals on our latest products. Shop now and save big!</p>
                <a href="/deal" target="_blank">Learn More</a>
            </div>
            <div><p class="regular-text">A test paragraph.</p></div>
            <div><p>Another test paragraph.</p></div>
        </body>
    </html>"#;

    let doc = dom_query::Document::from(contents);

    // Before sanitization:
    assert!(doc.select("div.ad-block").exists());
    assert_eq!(doc.select("div").length(), 3);
    assert_eq!(doc.select("p").length(), 3);

    policy.sanitize_document(&doc);
    // After sanitization, the `div.ad-block` element is removed because
    // its text content matched the pattern 3 times, which is considered too noisy.
    assert!(!doc.select("div.ad-block").exists());
    assert_eq!(doc.select("div").length(), 2);
    assert_eq!(doc.select("p").length(), 2);
    Ok(())
}
