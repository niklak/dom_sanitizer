use std::sync::mpsc::channel;
use std::sync::Arc;

use html5ever::local_name;

use dom_sanitizer::plugin_policy::preset;
use dom_sanitizer::plugin_policy::PluginPolicy;
use dom_sanitizer::Restrictive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let policy: PluginPolicy<Restrictive> = PluginPolicy::builder()
        // Allow table elements
        .exclude(preset::MatchLocalNames(vec![
            local_name!("table"),
            local_name!("tbody"),
            local_name!("tr"),
            local_name!("th"),
            local_name!("td"),
        ]))
        .remove(preset::MatchLocalName(local_name!("style")))
        // `html`, `head`, and `body` are always kept
        .build();

    dbg!(&policy);
    let shared_policy = Arc::new(policy);

    let (tx, rx) = channel();

    for _ in 0..4 {
        let policy = shared_policy.clone();
        let thread_tx = tx.clone();
        std::thread::spawn(move || {
            let contents: &str = include_str!("../test-pages/table.html");
            let doc = dom_query::Document::from(contents);
            policy.sanitize_document(&doc);
            thread_tx.send(doc).unwrap();
        });
    }
    drop(tx);

    for doc in rx {
        assert!(!doc.select("style").exists());
        assert!(doc.select("table tr > td").exists());
    }

    Ok(())
}
