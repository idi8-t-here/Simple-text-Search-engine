// benches/benchmark_search.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use runtime::{perform_search, Scope, SearchType, AppMessage};
use std::sync::mpsc::channel;

fn bench_search(c: &mut Criterion) {
    let term = black_box("the");

    c.bench_function("perform_search - Trie + Word", |b| {
        b.iter(|| {
            let (tx, _rx) = channel::<AppMessage>();
            let _ = perform_search(Scope::Words, SearchType::Prefix, term, tx);
        });
    });

    c.bench_function("perform_search - Trie + Line", |b| {
        b.iter(|| {
            let (tx, _rx) = channel::<AppMessage>();
            let _ = perform_search(Scope::Lines, SearchType::Prefix, term, tx);
        });
    });

    c.bench_function("perform_search - Suffix + Word", |b| {
        b.iter(|| {
            let (tx, _rx) = channel::<AppMessage>();
            let _ = perform_search(Scope::Words, SearchType::Suffix, term, tx);
        });
    });

    c.bench_function("perform_search - Suffix + Line", |b| {
        b.iter(|| {
            let (tx, _rx) = channel::<AppMessage>();
            let _ = perform_search(Scope::Lines, SearchType::Suffix, term, tx);
        });
    });

    c.bench_function("perform_search - NGram + Word", |b| {
        b.iter(|| {
            let (tx, _rx) = channel::<AppMessage>();
            let _ = perform_search(Scope::Words, SearchType::Contains, term, tx);
        });
    });

    c.bench_function("perform_search - NGram + Line", |b| {
        b.iter(|| {
            let (tx, _rx) = channel::<AppMessage>();
            let _ = perform_search(Scope::Lines, SearchType::Contains, term, tx);
        });
    });
}

criterion_group!(benches, bench_search);
criterion_main!(benches);
