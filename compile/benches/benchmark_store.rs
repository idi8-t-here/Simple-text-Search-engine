use criterion::{criterion_group, criterion_main, Criterion};
use compile::{process_data, Trees, Scope};

fn bench_trie_word(c: &mut Criterion) {
    c.bench_function("process_data - Trie + Word", |b| {
        b.iter(|| process_data(Trees::Trie, Scope::Word))
    });
}

fn bench_trie_line(c: &mut Criterion) {
    c.bench_function("process_data - Trie + Line", |b| {
        b.iter(|| process_data(Trees::Trie, Scope::Line))
    });
}

fn bench_suffix_word(c: &mut Criterion) {
    c.bench_function("process_data - Suffix + Word", |b| {
        b.iter(|| process_data(Trees::Suffix, Scope::Word))
    });
}

fn bench_suffix_line(c: &mut Criterion) {
    c.bench_function("process_data - Suffix + Line", |b| {
        b.iter(|| process_data(Trees::Suffix, Scope::Line))
    });
}

fn bench_ngram_word(c: &mut Criterion) {
    c.bench_function("process_data - NGram + Word", |b| {
        b.iter(|| process_data(Trees::NGramIndex, Scope::Word))
    });
}

fn bench_ngram_line(c: &mut Criterion) {
    c.bench_function("process_data - NGram + Line", |b| {
        b.iter(|| process_data(Trees::NGramIndex, Scope::Line))
    });
}

criterion_group!(
    benches,
    bench_trie_word,
    bench_trie_line,
    bench_suffix_word,
    bench_suffix_line,
    bench_ngram_word,
    bench_ngram_line
);
criterion_main!(benches);
