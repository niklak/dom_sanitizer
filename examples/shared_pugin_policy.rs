use std::sync::Arc;

use dom_sanitizer::plugin_policy::preset;
use dom_sanitizer::plugin_policy::PluginPolicy;
use dom_sanitizer::Restrictive;

use html5ever::local_name;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let policy: PluginPolicy<Restrictive> = PluginPolicy::builder()
        // Allow table elements
        .exclude(preset::MatchLocalNames(vec![
            local_name!("table"),
            local_name!("tr"),
            local_name!("th"),
            local_name!("td"),
        ]))
        .remove(preset::MatchLocalName(local_name!("style")))
        // `html`, `head`, and `body` are always kept
        .build();
        
    dbg!(&policy);
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
