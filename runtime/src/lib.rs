use bincode::config;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use unicode_segmentation::UnicodeSegmentation;

use levenshtein::levenshtein;

use data_structs::trees;

use trees::ngram::NGramIndex;
use trees::suffix::SuffixTree;
use trees::trie::Trie;

#[derive(Debug, Clone)]
pub enum Scope {
    Words,
    Lines,
}

#[derive(Clone)]
pub enum SearchIndex {
    Trie(Trie),
    SuffixTree(SuffixTree),
    NGramIndex(NGramIndex),
}

#[derive(Debug, Clone)]
pub enum SearchType {
    Prefix,
    Suffix,
    Contains,
}

pub enum AppMessage {
    SearchComplete(Vec<(u8, String)>, std::time::Duration),
    Debug(String),
}

pub fn perform_search(
    index: &HashMap<String, SearchIndex>,
    scope: Scope,
    search_type: SearchType,
    term: &str,
    debug_sender: Sender<AppMessage>,
) -> Vec<(u8, String)> {
    let mut sorted_result: Vec<(u8, String)> = Vec::new();
    let scope_path = match scope {
        Scope::Words => "word_scope",
        Scope::Lines => "line_scope",
    };

    let type_path = match search_type {
        SearchType::Prefix => "trie-serial.bin",
        SearchType::Suffix => "suffix-serial.bin",
        SearchType::Contains => "ngram-serial.bin",
    };

    let path = format!("./serialized_outputs/{}/{}", scope_path, type_path);
    //let file_path = Path::new(&path);

    if let Err(e) = debug_sender.send(AppMessage::Debug(format!("Searching in file: {}", path))) {
        eprintln!("Failed to send debug message: {}", e);
    }

    let message = match search_type {
        SearchType::Prefix => "TRIE decoded successfully".to_string(),
        SearchType::Suffix => "SUFFIX decoded successfully".to_string(),
        SearchType::Contains => "NGRAM decoded successfully".to_string(),
    };

    let results = match search_type {
        SearchType::Contains => match scope {
            Scope::Words => index.get("NGramIndex_Word").and_then(|idx| {
                if let SearchIndex::NGramIndex(ngram_index) = idx {
                    ngram_index
                        .search(term.to_string())
                        .ok()
                        .map(Some)
                        .unwrap_or(None)
                } else {
                    None
                }
            }),
            Scope::Lines => index.get("NGramIndex_Line").and_then(|idx| {
                if let SearchIndex::NGramIndex(ngram_index) = idx {
                    ngram_index
                        .search(term.to_string())
                        .ok()
                        .map(Some)
                        .unwrap_or(None)
                } else {
                    None
                }
            }),
        },
        SearchType::Suffix => match scope {
            Scope::Words => index.get("SuffixTree_Word").and_then(|idx| {
                if let SearchIndex::SuffixTree(suffix_tree) = idx {
                    suffix_tree
                        .search(term.to_string())
                        .ok()
                        .map(Some)
                        .unwrap_or(None)
                } else {
                    None
                }
            }),
            Scope::Lines => index.get("SuffixTree_Line").and_then(|idx| {
                if let SearchIndex::SuffixTree(suffix_tree) = idx {
                    suffix_tree
                        .search(term.to_string())
                        .ok()
                        .map(Some)
                        .unwrap_or(None)
                } else {
                    None
                }
            }),
        },
        SearchType::Prefix => match scope {
            Scope::Words => index.get("Trie_Word").and_then(|idx| {
                if let SearchIndex::Trie(trie) = idx {
                    trie.search(term.to_string()).ok().map(Some).unwrap_or(None)
                } else {
                    None
                }
            }),
            Scope::Lines => index.get("Trie_Line").and_then(|idx| {
                if let SearchIndex::Trie(trie) = idx {
                    trie.search(term.to_string()).ok().map(Some).unwrap_or(None)
                } else {
                    None
                }
            }),
        },
    };

    if let Some(results) = results {
        if let Err(e) = debug_sender.send(AppMessage::Debug("File read successfully".to_string())) {
            eprintln!("Failed to send debug message: {}", e);
        }
        if let Err(e) = debug_sender.send(AppMessage::Debug(message)) {
            eprintln!("Failed to send debug message: {}", e);
        }

        for item in results.iter() {
            if matches!(scope, Scope::Lines) {
                let lines_scope = item.unicode_words().collect::<Vec<&str>>();
                if let (Some(first_word), Some(last_word)) =
                    (lines_scope.first(), lines_scope.last())
                {
                    let condition = match search_type {
                        SearchType::Contains => {
                            *first_word.to_lowercase() != term.to_lowercase()
                                && *last_word.to_lowercase() != term.to_lowercase()
                        }
                        SearchType::Suffix => *last_word.to_lowercase() == term.to_lowercase(),
                        SearchType::Prefix => *first_word.to_lowercase() == term.to_lowercase(),
                    };
                    if condition {
                        let priority = levenshtein(term, item);
                        sorted_result.push((priority as u8, item.to_string()));
                    }
                }
            } else {
                let priority = levenshtein(term, item);
                sorted_result.push((priority as u8, item.to_string()));
            }
        }
    }

    sorted_result.sort();
    sorted_result.truncate(100);
    sorted_result
}

pub fn load_index() -> Result<HashMap<String, SearchIndex>, String> {
    let base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent() // go one directory up
        .ok_or("Failed to determine project root")?
        .to_path_buf();

    let paths = [
        ("Trie_Word", "serialized_outputs/word_scope/trie-serial.bin"),
        (
            "SuffixTree_Word",
            "serialized_outputs/word_scope/suffix-serial.bin",
        ),
        (
            "NGramIndex_Word",
            "serialized_outputs/word_scope/ngram-serial.bin",
        ),
        ("Trie_Line", "serialized_outputs/line_scope/trie-serial.bin"),
        (
            "SuffixTree_Line",
            "serialized_outputs/line_scope/suffix-serial.bin",
        ),
        (
            "NGramIndex_Line",
            "serialized_outputs/line_scope/ngram-serial.bin",
        ),
    ];

    let mut result = HashMap::new();

    for (key, relative_path) in paths.iter() {
        let full_path = base_path.join(relative_path);
        let contents =
            fs::read(&full_path).map_err(|_| format!("Failed to read file: {:?}", full_path))?;
        let decoded: SearchIndex = match *key {
            "Trie_Word" | "Trie_Line" => {
                let trie: Trie = bincode::decode_from_slice(&contents, config::standard())
                    .map_err(|_| format!("Failed to decode trie: {:?}", full_path))?
                    .0;
                SearchIndex::Trie(trie)
            }
            "SuffixTree_Word" | "SuffixTree_Line" => {
                let suffix_tree: SuffixTree =
                    bincode::decode_from_slice(&contents, config::standard())
                        .map_err(|_| format!("Failed to decode suffix tree: {:?}", full_path))?
                        .0;
                SearchIndex::SuffixTree(suffix_tree)
            }
            "NGramIndex_Word" | "NGramIndex_Line" => {
                let ngram_index: NGramIndex =
                    bincode::decode_from_slice(&contents, config::standard())
                        .map_err(|_| format!("Failed to decode ngram: {:?}", full_path))?
                        .0;
                SearchIndex::NGramIndex(ngram_index)
            }
            _ => return Err(format!("Unknown key: {}", key)),
        };
        result.insert(key.to_string(), decoded);
    }

    Ok(result)
}
