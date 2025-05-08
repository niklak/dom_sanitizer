use std::{thread, time::Duration};

use dom_sanitizer::RestrictivePolicy;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    thread::sleep(Duration::from_secs(10));
    let contents: &str = include_str!("../test-pages/rustwiki_2024.html");

    let doc = dom_query::Document::from(contents);

    let _ = contents;
    let policy = RestrictivePolicy::builder()
        .remove_elements(&["style"])
        .build();
    policy.sanitize_document(&doc);

    thread::sleep(Duration::from_secs(60));
    Ok(())
}
