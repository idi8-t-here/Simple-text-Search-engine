use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::mpsc::channel;
use runtime::{perform_search, Scope, SearchType, load_index}; // Replace `your_crate_name` with the actual crate name


fn benchmark_perform_search(c: &mut Criterion) {
    let index = load_index().expect("Failed to load index");
    let term = "me";
    let scopes = [Scope::Words, Scope::Lines];
    let search_types = [SearchType::Prefix, SearchType::Suffix, SearchType::Contains];

    for scope in &scopes {
        for search_type in &search_types {
            let bench_name = format!("perform_search_{:?}_{:?}", scope, search_type);
            let scope_clone = scope.clone();
            let search_type_clone = search_type.clone();
            let index_clone = index.clone();

            // Setup before benchmarking
            let (sender, _) = channel();
            let result_len = perform_search(
                &index_clone,
                scope_clone.clone(),
                search_type_clone.clone(),
                term,
                sender.clone(),
            )
            .len();

            // Print once before the benchmark
            println!("Result length for {:?}_{:?}: {}", scope, search_type, result_len);

            c.bench_function(&bench_name, move |b| {
                b.iter(|| {
                    let (sender_inner, _receiver_inner) = channel();
                    perform_search(
                        black_box(&index_clone),
                        black_box(scope_clone.clone()),
                        black_box(search_type_clone.clone()),
                        black_box(term),
                        black_box(sender_inner),
                    );
                });
            });
        }
    }
}


criterion_group!(benches, benchmark_perform_search);
criterion_main!(benches);
