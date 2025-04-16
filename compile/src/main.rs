use std::{fs::{self, File}, io::Write, time::Instant};
use bincode::config;
use unicode_segmentation::UnicodeSegmentation;
use std::path::Path;

use data_structs::trees;
use trees::trie::Trie;

use trees::suffix::SuffixTree;
use trees::suffix::SearchScopeSuffix;

use trees::ngram::NGramIndex;
use trees::ngram::SearchScopeNgram;

enum Trees {
    Trie,
    Suffix,
    NGramIndex,
}

enum Scope {
    Word,
    Line
}

fn process_data(trees: Trees, search_scope: Scope) {
    let dataset_path = Path::new("./Dataset/words.txt");
    let dataset = fs::read_to_string(dataset_path).unwrap();

    // Select dataset and set limits
    let (chosen_dataset, limit) = match search_scope {
        Scope::Word => (dataset.unicode_words().collect::<Vec<&str>>(), 255),
        Scope::Line => (dataset.split('\n').collect::<Vec<&str>>(), 32768),
    };

    // Initialize the appropriate data structure
    let serialized_output = match trees {
        Trees::Trie => {
            let mut trie = Trie::new();
            for token in chosen_dataset.iter() {
                if token.len() > limit { continue; }
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
                if token.len() > limit { continue; }
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
                if token.len() > limit { continue; }
                ngram.store(token.to_string());
            }
            bincode::encode_to_vec(ngram, config::standard()).unwrap()
        }
    };

    // Create output path and write file
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

    let mut serialized_file = File::create(file_path).unwrap();
    serialized_file.write_all(&serialized_output).unwrap();
}

fn main() {
    let now = Instant::now();
    process_data(Trees::Trie, Scope::Line);
    process_data(Trees::Trie, Scope::Word);

    process_data(Trees::Suffix, Scope::Line);
    process_data(Trees::Suffix, Scope::Word);

    process_data(Trees::NGramIndex, Scope::Line);
    process_data(Trees::NGramIndex, Scope::Word);
    let time_taken = now.elapsed().as_secs_f32();
    eprintln!("Time taken to process document - {}",time_taken);
}
