use dom_query::Document;
use dom_sanitizer::{AllowAllPolicy, DenyAllPolicy};
use dom_sanitizer::preset::{table_policy, highlight_policy};

#[test]
fn test_restrictive_policy_exclude_table_highlight() {
    // restrict all elements except table and highlight elements, using preset policies
    // restrict all attributes.
    let policy = DenyAllPolicy::builder()
        .merge(table_policy())
        .merge(highlight_policy())
        .build();

    let contents = include_str!("../test-pages/table.html");
    let doc = Document::from(contents);
    assert!(doc.select("h1").exists());
    assert!(doc.select("table").exists());
    let before_small_count = doc.select("small").length();
    let before_b_count = doc.select("b").length();
    policy.sanitize_document(&doc);
    assert!(!doc.select("h1").exists());
    assert!(doc.select("table").exists());
    assert_eq!(doc.select("small").length(), before_small_count);
    assert_eq!(doc.select("b").length(), before_b_count);
}

#[test]
fn test_permissive_policy_exclude_table_highlight() {
    // allow all elements except table and highlight elements, using preset policies.
    // allow all attributes.
    let policy = AllowAllPolicy::builder()
        .merge(table_policy())
        .merge(highlight_policy())
        .build();

    let contents = include_str!("../test-pages/table.html");
    let doc = Document::from(contents);
    assert!(doc.select("h1").exists());
    assert!(doc.select("table").exists());
    policy.sanitize_document(&doc);
    assert!(doc.select("h1").exists());
    assert!(!doc.select("table").exists());
    assert_eq!(doc.select("small").length(), 0);
    assert_eq!(doc.select("b").length(), 0);
}