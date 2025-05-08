use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

use dom_query::Document;
use dom_sanitizer::RestrictivePolicy;

fn sanitize_deny_all(contents: &str, policy: &RestrictivePolicy) {
    let doc = Document::from(contents);

    policy.sanitize_document(&doc);
}

fn bench_dom_query(c: &mut Criterion) {
    let contents = include_str!("../test-pages/rustwiki_2024.html");
    let policy = RestrictivePolicy::builder()
        .remove_elements(&["style"])
        .build();
    c.bench_function("sanitize_deny_all", |b| {
        b.iter(|| sanitize_deny_all(black_box(&contents), black_box(&policy)))
    });
}

criterion_group!(benches, bench_dom_query);
criterion_main!(benches);
