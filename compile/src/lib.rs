use bincode::config;
use std::path::Path;
use std::{
    fs::{self, File},
    io::Write,
};
use unicode_segmentation::UnicodeSegmentation;

use data_structs::trees;
use trees::ngram::{NGramIndex, SearchScopeNgram};
use trees::suffix::{SearchScopeSuffix, SuffixTree};
use trees::trie::Trie;

pub enum Trees {
    Trie,
    Suffix,
    NGramIndex,
}

pub enum Scope {
    Word,
    Line,
}

pub fn process_data(trees: Trees, search_scope: Scope) {
    let dataset_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap() // one level up to project root
        .join("Dataset/words.txt");
    let dataset = fs::read_to_string(dataset_path).unwrap();

    let (chosen_dataset, limit) = match search_scope {
        Scope::Word => (dataset.unicode_words().collect::<Vec<&str>>(), 255),
        Scope::Line => (dataset.lines().collect::<Vec<&str>>(), 32768),
    };

    let serialized_output = match trees {
        Trees::Trie => {
            let mut trie = Trie::new();
            for token in chosen_dataset.iter() {
                if token.len() > limit {
                    continue;
                }
                trie.store(token.to_string());
            }
            bincode::encode_to_vec(trie, config::standard()).unwrap()
        }
        Trees::Suffix => {
            let mut suffix = SuffixTree::new();
            if let Scope::Line = search_scope {
                suffix.search_type = SearchScopeSuffix::Lines;
            }
            for token in chosen_dataset.iter() {
                if token.len() > limit {
                    continue;
                }
                suffix.store(vec![token]);
            }
            bincode::encode_to_vec(suffix, config::standard()).unwrap()
        }
        Trees::NGramIndex => {
            let mut ngram = NGramIndex::new();
            if let Scope::Line = search_scope {
                ngram.search_type = SearchScopeNgram::Lines;
            }
            for token in chosen_dataset.iter() {
                if token.len() > limit {
                    continue;
                }
                ngram.store(token.to_string());
            }
            bincode::encode_to_vec(ngram, config::standard()).unwrap()
        }
    };

    let scope_path = match search_scope {
        Scope::Word => "word_scope",
        Scope::Line => "line_scope",
    };

    let type_path = match trees {
        Trees::Trie => "trie-serial.bin",
        Trees::Suffix => "suffix-serial.bin",
        Trees::NGramIndex => "ngram-serial.bin",
    };

    let path = format!("./serialized_outputs/{}/{}", scope_path, type_path);
    let file_path = Path::new(&path);

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    let mut serialized_file = File::create(file_path).unwrap();
    serialized_file.write_all(&serialized_output).unwrap();
}
