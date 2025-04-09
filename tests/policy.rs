use dom_query::Document;
use dom_sanitizer::{AllowAllPolicy, DenyAllPolicy};

static PARAGRAPH_CONTENTS: &str = r#"
<!DOCTYPE html>
<html>
    <head><title>Test</title></head>
    <body>
        <!--Paragraphs-->
        <div><p>The first paragraph contains <a href="/first">the first link</a>.</p></div>
        <div><p>The second paragraph contains <a href="/second">the second link</a>.</p></div>
        <div><p>The third paragraph contains <a href="/third">the third link</a>.</p></div>
    </body>
</html>"#;

#[test]
fn test_restrictive_policy() {
    let policy = DenyAllPolicy::builder()
        .exclude_elements(&["p", "a"])
        .build();
    let doc = Document::from(PARAGRAPH_CONTENTS);
    policy.sanitize_document(&doc);
    assert!(!doc.select("div").exists());
    assert_eq!(doc.select("p > a").length(), 3);
}

#[test]
fn test_permissive_policy() {
    let policy = AllowAllPolicy::builder()
        .exclude_elements(&["div"])
        .build();
    let doc = Document::from(PARAGRAPH_CONTENTS);
    policy.sanitize_document(&doc);
    assert!(!doc.select("div").exists());
    assert_eq!(doc.select("p > a").length(), 3);
}
