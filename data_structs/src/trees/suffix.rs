use bincode::{Decode, Encode};
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Decode, Encode)]
pub struct SuffixTree {
    children: Option<HashMap<String, SuffixTree>>,
    isterminal: bool,
    value: String,
    count: usize,
    pub search_type: SearchScopeSuffix,
    metadata: Option<HashMap<usize, String>>,
}

#[derive(Debug, Decode, Encode)]
pub enum SearchScopeSuffix {
    Words,
    Lines,
}

impl Default for SuffixTree {
    fn default() -> Self {
        Self::new()
    }
}

impl SuffixTree {
    pub fn new() -> Self {
        Self {
            count: 0,
            children: None,
            value: "root".to_string(),
            isterminal: false,
            search_type: SearchScopeSuffix::Words,
            metadata: Some(HashMap::new()),
        }
    }

    pub fn store(&mut self, key: Vec<&str>) {
        let node = self;
        node.count += 1;

        match node.search_type {
            SearchScopeSuffix::Words => {
                for item in key.iter() {
                    node.metadata
                        .as_mut()
                        .unwrap()
                        .insert(node.count, item.to_string());

                    let chars: Vec<char> = item.chars().collect();
                    for i in 0..chars.len() {
                        let suffix: String = chars[i..].iter().collect();
                        let formatted = format!("${}", node.count);
                        let full_suffix = suffix + &formatted;

                        node.children.get_or_insert(HashMap::new());
                        let children = node.children.as_mut().unwrap();

                        // Make key unique using suffix, count, and current children length
                        let unique_key = format!(
                            "{}_{}_{}",
                            full_suffix,
                            node.count,
                            children.len() // Add position to make truly unique
                        );

                        children.insert(
                            unique_key,
                            SuffixTree {
                                children: None,
                                count: node.count,
                                isterminal: true,
                                value: full_suffix,
                                metadata: None,
                                search_type: SearchScopeSuffix::Words,
                            },
                        );
                    }
                }
            }

            SearchScopeSuffix::Lines => {
                let first = key.first().unwrap();
                node.metadata
                    .as_mut()
                    .unwrap()
                    .insert(node.count, first.to_string());

                let words: Vec<&str> = first.unicode_words().collect();
                let formatted = format!("${}", node.count);

                for i in 0..=words.len() {
                    let suffix = words[i..].join(" ") + &formatted;
                    node.children.get_or_insert(HashMap::new());

                    let children = node.children.as_mut().unwrap();

                    if let Some(child) = children
                        .get_mut(&suffix.split_whitespace().next().unwrap_or("").to_string())
                    {
                        let suffix_words: Vec<&str> = suffix.split_whitespace().collect();
                        let child_words: Vec<&str> = child.value.split_whitespace().collect();

                        let common_words: Vec<&str> = suffix_words
                            .iter()
                            .zip(child_words.iter())
                            .take_while(|(a, b)| a == b)
                            .map(|(w, _)| *w)
                            .collect();

                        if !common_words.is_empty() {
                            let common_prefix = common_words.join(" ");
                            let suffix_remaining = suffix[common_prefix.len()..].trim().to_string();
                            let child_remaining =
                                child.value[common_prefix.len()..].trim().to_string();

                            child.value = common_prefix;
                            child.children.get_or_insert(HashMap::new());
                            let child_children = child.children.as_mut().unwrap();

                            if !suffix_remaining.is_empty() {
                                child_children.insert(
                                    suffix_remaining
                                        .split_whitespace()
                                        .next()
                                        .unwrap_or("")
                                        .to_string(),
                                    SuffixTree {
                                        children: None,
                                        count: node.count,
                                        isterminal: true,
                                        metadata: None,
                                        value: suffix_remaining,
                                        search_type: SearchScopeSuffix::Lines,
                                    },
                                );
                            }

                            if !child_remaining.is_empty() {
                                child_children.insert(
                                    child_remaining
                                        .split_whitespace()
                                        .next()
                                        .unwrap_or("")
                                        .to_string(),
                                    SuffixTree {
                                        children: None,
                                        isterminal: true,
                                        metadata: None,
                                        count: node.count - 1,
                                        value: child_remaining,
                                        search_type: SearchScopeSuffix::Lines,
                                    },
                                );
                            }

                            child.isterminal = false;
                        }
                    } else {
                        children.insert(
                            suffix.split_whitespace().next().unwrap_or("").to_string(),
                            SuffixTree {
                                children: None,
                                count: node.count,
                                isterminal: true,
                                metadata: None,
                                value: suffix,
                                search_type: SearchScopeSuffix::Lines,
                            },
                        );
                    }
                }
            }
        }
    }

    pub fn search(&self, key: &str) -> Result<Vec<&str>, &str> {
        let node = self;
        let mut results = Vec::new();

        if let Some(children) = &node.children {
            for (_, child) in children.iter() {
                // Remove the terminal marker for comparison
                let value_without_terminal: String =
                    child.value.chars().take_while(|&c| c != '$').collect();

                // Check if this suffix ends with our key
                if value_without_terminal.ends_with(key) {
                    if let Some(metadata) = &node.metadata {
                        if let Some(checking) = metadata.get(&child.count) {
                            // Only exclude exact matches with the search key
                            if checking != key {
                                // Add all instances, including duplicates from different counts
                                results.push(checking.as_str());
                            }
                        }
                    }
                }
            }
        }

        if results.is_empty() {
            Err("No results found")
        } else {
            // Sort to group duplicates together (optional)
            results.sort();
            Ok(results)
        }
    }
}
