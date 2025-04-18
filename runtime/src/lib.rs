use std::path::Path;
use std::fs;
use unicode_segmentation::UnicodeSegmentation;
use bincode::config;
use std::sync::mpsc::Sender;

use levenshtein::levenshtein;

use data_structs::trees;

use trees::ngram::NGramIndex;
use trees::suffix::SuffixTree;
use trees::trie::Trie;

#[derive(Debug)]
pub enum Scope {
    Words,
    Lines,
}

#[derive(Debug)]
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
    let file_path = Path::new(&path);

    debug_sender
        .send(AppMessage::Debug(format!("Searching in file: {}", path)))
        .unwrap();
    let message = match search_type {
        SearchType::Prefix => "TRIE decoded successfully".to_string(),
        SearchType::Suffix => "SUFFIX decoded successfully".to_string(),
        SearchType::Contains => "NGRAM decoded successfully".to_string(),
    };

    if let Ok(contents) = fs::read(file_path) {
        let results = match search_type {
            SearchType::Contains => {
                match bincode::decode_from_slice::<NGramIndex, _>(&contents, config::standard()) {
                    Ok((tree, _)) => match tree.search(term.to_string()) {
                        Ok(search_results) => search_results,
                        Err(e) => {
                            debug_sender.send(AppMessage::Debug(e.to_string())).unwrap();
                            return Vec::new();
                        }
                    },
                    Err(e) => {
                        debug_sender.send(AppMessage::Debug(e.to_string())).unwrap();
                        return Vec::new();
                    }
                }
            }
            SearchType::Suffix => {
                match bincode::decode_from_slice::<SuffixTree, _>(&contents, config::standard()) {
                    Ok((tree, _)) => match tree.search(term) {
                        Ok(search_results) => {
                            search_results.iter().map(|x| x.to_string()).collect()
                        }
                        Err(e) => {
                            debug_sender.send(AppMessage::Debug(e.to_string())).unwrap();
                            return Vec::new();
                        }
                    },
                    Err(e) => {
                        debug_sender.send(AppMessage::Debug(e.to_string())).unwrap();
                        return Vec::new();
                    }
                }
            }
            SearchType::Prefix => {
                match bincode::decode_from_slice::<Trie, _>(&contents, config::standard()) {
                    Ok((tree, _)) => match tree.search(term.to_string()) {
                        Ok(search_results) => search_results,
                        Err(e) => {
                            debug_sender.send(AppMessage::Debug(e.to_string())).unwrap();
                            return Vec::new();
                        }
                    },
                    Err(e) => {
                        debug_sender.send(AppMessage::Debug(e.to_string())).unwrap();
                        return Vec::new();
                    }
                }
            }
        };

        debug_sender
            .send(AppMessage::Debug("File read successfully".to_string()))
            .unwrap();
        debug_sender.send(AppMessage::Debug(message)).unwrap();

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
                        debug_sender.send(AppMessage::Debug(format!(
                            "MATCH: word='{}', term='{}', full_line='{}'",
                            if matches!(search_type, SearchType::Prefix) { first_word } else { last_word },
                            term,
                            item
                        ))).unwrap();
                        let priority = levenshtein(term, item);
                        sorted_result.push((priority as u8, item.to_string()));
                    } else {
                        debug_sender.send(AppMessage::Debug(format!(
                            "FAILED: word='{}', term='{}', full_line='{}'",
                            if matches!(search_type, SearchType::Prefix) { first_word } else { last_word },
                            term,
                            item
                        ))).unwrap();
                    }
                } else {
                    debug_sender.send(AppMessage::Debug(
                        format!("Empty line or no words found in: '{}'", item)
                    )).unwrap();
                }
            } else {
                let priority = levenshtein(term, item);
                sorted_result.push((priority as u8, item.to_string()));
            }
        }
    } else {
        debug_sender
            .send(AppMessage::Debug("Failed to read file".to_string()))
            .unwrap();
    }

    sorted_result.sort();
    sorted_result.truncate(100);
    sorted_result
}
