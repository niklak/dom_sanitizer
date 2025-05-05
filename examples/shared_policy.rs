use std::sync::Arc;

use dom_sanitizer::preset::table_policy;
use dom_sanitizer::DenyAllPolicy;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let policy = DenyAllPolicy::builder()
        // Allow table elements
        .merge(table_policy())
        .remove_elements(&["style"])
        // `html`, `head`, and `body` are always kept
        .build();
        
    let shared_policy = Arc::new(policy);

    for _ in 0..4 {
        let policy = shared_policy.clone();
        std::thread::spawn(move || {
            let contents: &str = include_str!("../test-pages/table.html");
            let doc = dom_query::Document::from(contents);
            policy.sanitize_document(&doc);
            assert!(doc.select("table > tr > td").exists());
            assert!(!doc.select("style").exists());
        });
    }

    Ok(())
}
