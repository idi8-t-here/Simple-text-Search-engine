use std::collections::{HashMap, HashSet};
use unicode_segmentation::UnicodeSegmentation;
use bincode::{Encode,Decode};

#[derive(Encode, Decode, Debug)]
pub struct NGramIndex {
    grams: Option<HashMap<Vec<String>, Vec<usize>>>,
    words: Option<Vec<String>>,
    gram_size: usize,
    pub search_type: SearchScope,
}

#[derive(Encode, Decode, Debug)]
pub enum SearchScope {
    Words,
    Lines,
}

impl Default for NGramIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl NGramIndex {

    pub fn new() -> Self {
        Self { 
            grams: None, 
            words: None, 
            gram_size: 2, 
            search_type: SearchScope::Words
        }
    }

    pub fn store(&mut self, key: String) {
        let ngram = self;
        let line_segment = key.unicode_words().map(|x| x.to_string()).collect::<Vec<String>>();

        let key_length = match ngram.search_type {
            SearchScope::Words => {
                // For words, we need to count graphemes (visible characters)
                key.graphemes(true).count()
            },
            SearchScope::Lines => line_segment.len(),
        };

        if ngram.words.is_none() {
            let mut new_hash = HashMap::new();

            for index in (ngram.gram_size - 1)..key_length {
                match ngram.search_type {
                    SearchScope::Words => {
                        // Get graphemes properly
                        let graphemes: Vec<&str> = key.graphemes(true).collect();
                        if index + ngram.gram_size > graphemes.len() {
                            continue;
                        }
                        let word_segment = graphemes[index..index + ngram.gram_size].join("");
                        let keys: Vec<String> = vec![word_segment];
                        new_hash.insert(keys.clone(), vec![0]);
                    },
                    SearchScope::Lines => {
                        if index + ngram.gram_size > line_segment.len() {
                            continue;
                        }
                        let keys: Vec<String> = line_segment[index..index + ngram.gram_size].to_vec();
                        new_hash.insert(keys, vec![0]);
                    },
                };
            }
            ngram.grams = Some(new_hash);
            ngram.words = Some(vec![key]);
        } else {
            for index in (ngram.gram_size - 1)..key_length {
                match ngram.search_type {
                    SearchScope::Words => {
                        let graphemes: Vec<&str> = key.graphemes(true).collect();
                        if index + ngram.gram_size > graphemes.len() {
                            continue;
                        }
                        let word_segment = graphemes[index..index + ngram.gram_size].join("");
                        let keys: Vec<String> = vec![word_segment];
                        ngram.grams.as_mut().unwrap()
                            .entry(keys)
                            .and_modify(|v| v.push(ngram.words.as_ref().unwrap().len()))
                            .or_insert(vec![ngram.words.as_ref().unwrap().len()]);
                    },
                    SearchScope::Lines => {
                        if index + ngram.gram_size > line_segment.len() {
                            continue;
                        }
                        let keys: Vec<String> = line_segment[index..index + ngram.gram_size].to_vec();
                        ngram.grams.as_mut().unwrap()
                            .entry(keys)
                            .and_modify(|v| v.push(ngram.words.as_ref().unwrap().len()))
                            .or_insert(vec![ngram.words.as_ref().unwrap().len()]);
                    },
                }
            }
            ngram.words.as_mut().unwrap().push(key);
        }
    }

    pub fn search(&self, key: String) -> Result<Option<Vec<String>>, &str> {
        let ngram = self;
        let mut results = HashSet::new(); 

        for (key_in_gram, values) in ngram.grams.as_ref().unwrap().iter() {
            let condition = match ngram.search_type {
                SearchScope::Words => key_in_gram.iter().any(|x| x.contains(&key)),
                SearchScope::Lines => key_in_gram.contains(&key),
            };
            if condition {
                for value in values.iter() {
                    if let Some(word) = ngram.words.as_ref().unwrap().get(*value) {
                        if !word.starts_with(&key) && !word.ends_with(&key) && word.contains(&key) {
                            results.insert(word.clone()); // HashSet automatically handles duplicates
                        }
                    }
                }
            }
        }

        if results.is_empty() {
            Err("couldn't find a match mate")
        } else {
            Ok(Some(results.into_iter().collect())) 
        }
    }
}
