## Simple Text Search Engine

A blazing-fast, modular text search engine designed to support different search strategies (prefix, suffix, contains) and flexible tokenization scopes (word-level, line-level).  
The engine leverages advanced data structures and popular Rust crates to efficiently index and search through large datasets — ranking results based on similarity and relevance.

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
for first time run, this runs both the compile and runtime app on after another(where the compile generates the indexes), but after first time run we can just use:
```bash
cargo run
```

## 📦 Implementation Details

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

---

## Problem Breakdown

### SEARCH_SCOPE

An enum that defines the level of tokenization for searching.  
Options:  
- Words (i.e. tokenizing by characters)
- Lines (i.e. tokenizing by words)

**Solution**  
To handle scope-based tokenization efficiently, we use the [Unicode Segmentation](https://crates.io/crates/unicode-segmentation) crate. It ensures proper segmentation of words and lines, respecting Unicode boundaries.

### SEARCH_TYPE

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

## ⚙️ How It Runs

1. **Build Phase:**  
   Pre-runtime code to process and serialize the dataset into three optimized data structures (Radix Tree, Suffix Tree, N gram) using `bincode`.

2. **Runtime Phase:**  
   User is prompted to select:
   - A `SEARCH_SCOPE` (words or lines)
   - A `SEARCH_TYPE` (prefix, suffix, contains)

3. **Search & Rank:**  
   The engine tokenizes the user’s query based on the selected scope, performs the search based on the selected type, and returns results sorted by rank using Levenshtein distance.

## Coming Soon
- [ ] partitioning in order to avoid reserializing the whole dataset again
