use dom_query::Document;
use dom_sanitizer::{AllowAllPolicy, DenyAllPolicy, SanitizeExt};

mod data;

use data::PARAGRAPH_CONTENTS;

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
    assert_eq!(doc.select("[role]").length(), 7);
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
    assert_eq!(doc.select("p").length(), 4);
    assert_eq!(doc.select("p[role]").length(), 0);
    assert_eq!(doc.select("p > a[href][role]").length(), 3);
}

#[test]
fn test_restrictive_policy_simple() {
    let policy = DenyAllPolicy::builder().build();
    let doc = Document::from(PARAGRAPH_CONTENTS);
    doc.root().sanitize(&policy);
    assert!(!doc.select("div").exists());
    assert!(!doc.select("p").exists());
    assert!(!doc.select("a").exists());

    assert!(doc.select("html").exists());
    assert!(doc.select("head").exists());

    let body_sel = doc.select("body");
    assert_eq!(body_sel.length(), 1);
    let body_node = body_sel.nodes().first().unwrap();

    // only one combined (normalized) text node is left
    assert_eq!(body_node.descendants().len(), 1);
}

#[test]
fn test_permissive_policy_simple() {
    let policy = AllowAllPolicy::builder().build();
    let doc = Document::from(PARAGRAPH_CONTENTS);
    doc.sanitize(&policy);

    assert!(doc.select("html").exists());
    assert!(doc.select("head").exists());
    assert!(doc.select("body").exists());
    assert!(doc.select("div").exists());
    assert!(doc.select("p").exists());
    assert!(doc.select("a").exists());
}

#[test]
fn test_permissive_policy_remove() {
    // In some cases it's not enough to just exclude elements from the sanitization policy.
    // You may want to remove them from the DOM tree entirely, including their children.
    // E.g. when you want to remove all `style` elements from the document.
    // But you specify it in exclude_elements() method, it will only remove the `style` element, keeping its text content.
    let policy = AllowAllPolicy::builder()
        .exclude_elements(&["style"])
        .build();
    let contents = include_str!("../test-pages/table.html");
    let doc = Document::from(contents);
    policy.sanitize_document(&doc);

    assert!(!doc.select("style").exists());

    // After sanitization, we got style inner contents without the `style` elements -- as a text node.
    assert!(doc.html().contains("border-collapse: collapse"));

    // For such cases it's better to use the `remove_elements()` method.
    let policy = AllowAllPolicy::builder()
        .remove_elements(&["style"])
        .build();
    let doc = Document::from(contents);
    policy.sanitize_document(&doc);
    // in that case style elements are removed from the DOM tree, including their text content.
    assert!(!doc.select("style").exists());
    assert!(!doc.html().contains("border-collapse: collapse"));
}

#[test]
fn test_restrictive_policy_remove() {
    // Removing elements with `DenyAllPolicy` works the same way as with `AllowAllPolicy`.

    let policy = DenyAllPolicy::builder().remove_elements(&["style"]).build();
    let doc: Document = include_str!("../test-pages/table.html").into();

    policy.sanitize_document(&doc);
    // in that case style elements are removed from the DOM tree, including their text content.
    assert!(!doc.select("style").exists());
    assert!(!doc.html().contains("border-collapse: collapse"));
}

#[test]
fn test_restrictive_policy_remove_html() {
    // Removing elements with `DenyAllPolicy` works the same way as with `AllowAllPolicy`.

    let policy = AllowAllPolicy::builder()
        .remove_elements(&["style"])
        .build();
    let contents = include_str!("../test-pages/table.html");
    assert!(contents.contains("border-collapse: collapse"));
    assert!(contents.contains("<style>"));
    let html = policy.sanitize_html(contents);

    assert!(!html.contains("<style>"));
    assert!(!html.contains("border-collapse: collapse"));
}

#[test]
fn test_restrictive_selection() {
    let policy = DenyAllPolicy::builder().build();
    let doc = Document::from(PARAGRAPH_CONTENTS);
    let sel = doc.select("p");
    assert!(!doc.select("p:only-text").exists());

    sel.sanitize(&policy);
    //policy.sanitize_selection(&doc.select("p"));

    assert_eq!(doc.select("p:only-text").length(), 4);
}
