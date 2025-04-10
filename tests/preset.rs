use dom_query::Document;
use dom_sanitizer::{AllowAllPolicy, DenyAllPolicy, Policy, Restrictive};
use dom_sanitizer::preset::{global_attr_policy, highlight_policy, list_policy, table_attr_policy, table_policy};

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

#[test]
fn test_restrictive_policy_exclude_table_attrs() {
    // restrict all elements except table elements, using preset policies.
    // restrict all attributes, except table attributes and global attributes (class, id, etc.).
    let policy: Policy<Restrictive> = Policy::builder()
        .merge(table_attr_policy())
        .merge(table_policy())
        .merge(global_attr_policy())
        .build();

    let contents = include_str!("../test-pages/table.html");
    let doc = Document::from(contents);
    assert!(doc.select("[data-nutrition]").exists());
    let before_th_scope_count = doc.select("th[scope]").length();
    let before_tr_nutrition_count = doc.select("tr.nutrition-item").length();

    policy.sanitize_document(&doc);
    assert!(!doc.select("[data-nutrition]").exists());
    assert_eq!(doc.select("th[scope]").length(), before_th_scope_count);
    assert_eq!(doc.select("tr.nutrition-item").length(), before_tr_nutrition_count);
}

#[test]
fn test_restrictive_policy_exclude_list() {
    // restrict all elements except list elements, using preset policies.
    // also allow heading elements h1 and h3.
    let policy: Policy<Restrictive> = Policy::builder()
        .merge(list_policy())
        .exclude_elements(&["h1", "h3"])
        .build();

    let contents = include_str!("../test-pages/list.html");
    let doc = Document::from(contents);
    let before_li_count = doc.select("ul > li").length();
    assert!(doc.select("h1").exists());
    assert!(doc.select("h3").exists());
    
    assert!(doc.select("mark").exists());
    assert!(doc.select("i").exists());
    assert!(doc.select("b").exists());

    policy.sanitize_document(&doc);
    assert_eq!(doc.select("ul > li").length(), before_li_count);

    assert!(doc.select("h1").exists());
    assert!(doc.select("h3").exists());

    assert!(!doc.select("mark").exists());
    assert!(!doc.select("i").exists());
    assert!(!doc.select("b").exists());
}