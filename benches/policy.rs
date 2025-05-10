use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

use dom_query::Document;
use dom_sanitizer::plugin_policy::{preset, RestrictivePluginPolicy};
use dom_sanitizer::RestrictivePolicy;

fn sanitize_restrictive_policy(contents: &str, policy: &RestrictivePolicy) {
    let doc = Document::from(contents);

    policy.sanitize_document(&doc);
}

fn sanitize_restrictive_plugin_policy(contents: &str, policy: &RestrictivePluginPolicy) {
    let doc = Document::from(contents);

    policy.sanitize_document(&doc);
}

fn bench_restrictive(c: &mut Criterion) {
    let contents = include_str!("../test-pages/rustwiki_2024.html");
    let policy = RestrictivePolicy::builder()
        .exclude_elements(&[
            "b", "del", "em", "i", "ins", "mark", "s", "small", "strong", "u",
        ])
        .remove_elements(&["style"])
        .build();
    c.bench_function("restrictive_policy", |b| {
        b.iter(|| sanitize_restrictive_policy(black_box(contents), black_box(&policy)))
    });
}

fn bench_restrictive_plugin_policy(c: &mut Criterion) {
    let contents = include_str!("../test-pages/rustwiki_2024.html");
    let policy = RestrictivePluginPolicy::builder()
        .remove(preset::LocalNamesMatcher::new(&[
            "b", "del", "em", "i", "ins", "mark", "s", "small", "strong", "u",
        ]))
        .remove(preset::LocalNameMatcher::new("style"))
        .build();
    c.bench_function("restrictive_plugin_policy", |b| {
        b.iter(|| sanitize_restrictive_plugin_policy(black_box(contents), black_box(&policy)))
    });
}

criterion_group!(benches, bench_restrictive, bench_restrictive_plugin_policy);
criterion_main!(benches);
