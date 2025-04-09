use dom_query::Document;
use dom_sanitizer::{AllowAllPolicy, DenyAllPolicy};

static PARAGRAPH_CONTENTS: &str = r#"
<!DOCTYPE html>
<html>
    <head><title>Test</title></head>
    <body>
        <!--Paragraphs-->
        <div><p role="paragraph">The first paragraph contains <a href="/first" role="link">the first link</a>.</p></div>
        <div><p role="paragraph">The second paragraph contains <a href="/second" role="link">the second link</a>.</p></div>
        <div><p role="paragraph">The third paragraph contains <a href="/third" role="link">the third link</a>.</p></div>
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

    assert!(doc.select("html").exists());
    assert!(doc.select("head").exists());
    assert!(doc.select("body").exists());
}

#[test]
fn test_permissive_policy() {
    let policy = AllowAllPolicy::builder().exclude_elements(&["div"]).build();
    let doc = Document::from(PARAGRAPH_CONTENTS);
    policy.sanitize_document(&doc);
    assert!(!doc.select("div").exists());
    assert_eq!(doc.select("p > a").length(), 3);

    // html, head, body are always kept
    assert!(doc.select("html").exists());
    assert!(doc.select("head").exists());
    assert!(doc.select("body").exists());
}

#[test]
fn test_restrictive_policy_attrs() {
    let policy = DenyAllPolicy::builder()
        .exclude_elements(&["p", "a"])
        .exclude_attrs(&["role"])
        .exclude_element_attrs("a", &["href"])
        .build();
    let doc = Document::from(PARAGRAPH_CONTENTS);
    policy.sanitize_document(&doc);
    assert!(!doc.select("div").exists());
    assert_eq!(doc.select("p > a[href]").length(), 3);
    assert_eq!(doc.select("[role]").length(), 6);
}

#[test]
fn test_permissive_policy_attrs() {
    let policy = AllowAllPolicy::builder()
        .exclude_elements(&["div"])
        .exclude_element_attrs("p", &["role"])
        .build();
    let doc = Document::from(PARAGRAPH_CONTENTS);
    policy.sanitize_document(&doc);
    assert!(!doc.select("div").exists());
    assert_eq!(doc.select("p").length(), 3);
    assert_eq!(doc.select("p[role]").length(), 0);
    assert_eq!(doc.select("p > a[href][role]").length(), 3);
}
