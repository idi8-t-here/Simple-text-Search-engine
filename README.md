## Simple Text Search Engine

A blazing-fast, modular text search engine designed to support different search strategies (prefix, suffix, contains) and flexible tokenization scopes (word-level, line-level).  
The engine leverages advanced data structures and popular Rust crates to efficiently index and search through large datasets — ranking results based on similarity and relevance.

## Benchmarks

### Search Performance

We conducted extensive benchmarking of our search implementations against Tantivy, a popular Rust search engine. All benchmarks were performed on a dataset of 466,550 words, searching for the term "the".

#### Our Implementation

| Search Type | Mean Results(min-avg-max) |
| -------------- | --------------- |
| Trie + Word: | ( 931.29 ns - **935.72 ns** - 940.80 ns ) (~0.94 µs) |
| Trie + Line: | ( 963.35 ns - **967.41 ns** - 971.61 ns ) (~0.97 µs) |
| Suffix + Word: | ( 1.0018 µs - **1.0047 µs** - 1.0077 µs ) |
| Suffix + Line: | ( 1.0050 µs - **1.0075 µs** - 1.0100 µs )|
|NGram + Word: |  ( 981.57 ns - **984.63 ns** - 987.78 ns ) (~0.98 µs)|
|NGram + Line: | ( 1.0078 µs - **1.0101 µs** - 1.0126 µs )|

#### Tantivy

| Search Type | Mean Results(min-avg-max) |
| -------------- | --------------- |
| Term - Search: | ( 18.833 µs - **18.877 µs** - 18.959 µs )  |

> [!NOTE] Note<br/>
> Our specialized search implementations consistently outperform Tantivy by a significant margin, being approximately 19x faster. 
> While our implementation shows superior performance for this specific use case, Tantivy is a full-featured search engine with additional capabilities beyond simple term searching. These benchmarks focus solely on search speed for single-term queries.

All benchmarks were conducted using Criterion.rs with 100 samples per measurement, including warm-up periods and statistical analysis for reliable results.

## Features

- Supports **prefix**, **suffix**, and **contains** based searches  
- Tokenization by **words** or **lines** using Unicode-aware segmentation  
- **Levenshtein distance** scoring (the same as tantivy, and meilisearch)
- Serialization of processed dataset for faster lookups at runtime  
- A **Ratatui** TUI support for seamless interaction

## Getting started
In order to run this text search engine we need to enter:
```bash
cargo run -p app
```
for first time run, this runs both the compile and runtime app on after another (where the compile generates the indexes), but after first time run we can just use this to run the runtime app:
```bash
cargo run
```

### Key Findings:
- **Fastest Implementation**: Trie + Word search at 935.72 ns
- **Slowest Implementation**: NGram + Line search at 1.0101 µs
- **Consistency**: All our search variants perform within a tight range (935-1010 ns)
- **Comparison**: Our slowest implementation (1.01 µs) is still 18.7x faster than Tantivy (18.88 µs)

## Implementation Details

### Resources Used

| List of resources we will use | Why? |
| ------------- | ---|
| [Trie tree wiki](https://en.wikipedia.org/wiki/Trie) | - For PREFIX_SEARCH Implemetation  |
| [Suffix tree wiki](https://en.m.wikipedia.org/wiki/Suffix_tree) | - For SUFFIX_SEARCH Implemetation |
| [N-gram wiki](https://en.wikipedia.org/wiki/N-gram) | - For CONTAINS_SEARCH Implemetation |

### Crates Used

| List of crates we will use | Why? |
| ------------- |---|
| [Unicode Segmentation](https://crates.io/crates/unicode-segmentation) | - For helping with search scope i.e Tokenization of words or lines |
| [Levenshtein](https://crates.io/crates/levenshtein)  | - For dictating the method by which we Rank search results |
| [thiserror](https://crates.io/crates/thiserror)  | - For custom error definitions in codebase |
| [bincode](https://crates.io/crates/bincode)  | - For processing dataset into binary  |
| [Ratatui](https://crates.io/crates/ratatui)  | - For augmenting UI experience |

## Problem Breakdown

#### SEARCH_SCOPE

An enum that defines the level of tokenization for searching.  
Options:  
- Words (i.e. tokenizing by characters)
- Lines (i.e. tokenizing by words)

**Solution**  
To handle scope-based tokenization efficiently, we use the [Unicode Segmentation](https://crates.io/crates/unicode-segmentation) crate. It ensures proper segmentation of words and lines, respecting Unicode boundaries.

#### SEARCH_TYPE

An enum that defines the type of search to be conducted.  
Options:  
- Prefix
- Suffix
- Contains

**Solution**  
Each type of search is supported by a specialized data structure:

- **Prefix Search** → [Radix Tree](https://en.wikipedia.org/wiki/Radix_tree)  
- **Suffix Search** → [Suffix Tree](https://en.m.wikipedia.org/wiki/Suffix_tree)  
- **Contains Search** → [N-gram(digrams..by default)](https://en.wikipedia.org/wiki/N-gram)

## How It Runs

1. **Build Phase:**  
   Pre-runtime app to process and serialize the dataset into three optimized data structures (Radix Tree, Suffix Tree, N gram) using `bincode`.

2. **Runtime Phase:**  
   User is prompted to select:
   - A `SEARCH_SCOPE` (words or lines)
   - A `SEARCH_TYPE` (prefix, suffix, contains)

3. **Search & Rank:**  
   The engine tokenizes the user’s query based on the selected scope, performs the search based on the selected type, and returns results sorted by rank using Levenshtein distance.

## Coming Soon
- [ ] partitioning in order to avoid reserializing the whole dataset again
