
![prefix-ezgif com-video-to-gif-converter](https://github.com/user-attachments/assets/a6a7d066-4062-4361-a50e-fc0494483c30)
![contains-ezgif com-video-to-gif-converter](https://github.com/user-attachments/assets/1e6b49b0-965d-494c-beae-ccd7076a32fb)
![suffix-ezgif com-video-to-gif-converter](https://github.com/user-attachments/assets/c95d0f33-3c62-4b58-92a6-2a03cda75b52)

## Simple Text Search Engine

A blazing-fast, modular text search engine designed to support different search strategies (prefix, suffix, contains) and flexible tokenization scopes (word-level, line-level).  
The engine leverages advanced data structures and popular Rust crates to efficiently index and search through large datasets — ranking results based on similarity and relevance.

## Getting started
In order to run this text search engine we need to enter:
```bash
cargo run -p app
```
for first time run, this runs both the compile and runtime app on after another (where the compile generates the indexes), but after first time run we can just use this to run the runtime app:
```bash
cargo run
```

## Benchmarks

### Search Performance

We conducted extensive benchmarking of our search implementations against Tantivy, a popular Rust search engine. All benchmarks were performed on a dataset of 466,550 words, searching for the term "me".

#### Our Implementation

| Search Type | Mean Results(min-avg-max) | Amount Matched (Before Truncating to top 100) |
| -------------- | --------------- |---|
| Trie + Word: | ( 819.85 ns - **820.58 ns** - 821.33 ns ) | 3970 |
| Trie + Line: | ( 934.21 ns - **935.14 ns** - 936.26 ns ) | 3 |
| Suffix + Word: | ( 439.63 µs - **440.26 µs** - 440.89 µs ) | 1926 |
| Suffix + Line: | ( 365.22 µs - **366.21 µs** - 367.39 µs )| 4 |
|NGram + Word: |  ( 4.5363 ms - **4.5548 ms** - 4.5745 ms ) | 14358 |
|NGram + Line: | ( [85.216 µs - **85.644 µs** - 86.327 µs )| 15 |

All benchmarks were conducted using Criterion.rs with 100 samples per measurement, including warm-up periods and statistical analysis for reliable results.

## Features

- Supports **prefix**, **suffix**, and **contains** based searches  
- Tokenization by **words** or **lines** using Unicode-aware segmentation  
- **Levenshtein distance** scoring (the same as tantivy, and meilisearch)
- Serialization of processed dataset for faster lookups at runtime  
- A **Ratatui** TUI support for seamless interaction

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
| [Trie tree wiki](https://en.wikipedia.org/wiki/Trie) (Reverse String Matching) | - For SUFFIX_SEARCH Implemetation |
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

- **Prefix Search** → [Trie tree wiki](https://en.wikipedia.org/wiki/Trie)  
- **Suffix Search** → [Trie tree wiki](https://en.wikipedia.org/wiki/Trie) (Reverse String Matching) 
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
