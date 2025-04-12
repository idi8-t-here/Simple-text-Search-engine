use std::{fs::{self, File}, io::Write};
use bincode::config;
use unicode_segmentation::UnicodeSegmentation;
use std::path::Path;

use data_structs::trees;
use trees::trie::Trie;

fn main() {
    let dataset_path = Path::new("./Dataset/output.txt");

    let dataset = fs::read_to_string(dataset_path).unwrap();
    let dataset_lines = dataset.split('\n').collect::<Vec<&str>>();
    let dataset_words = dataset.unicode_words().collect::<Vec<&str>>();

    let mut trie_lines = Trie::new();
    for lines in dataset_lines {
        if lines.len() > 32768 { continue; }  
        trie_lines.store(lines.to_string());
    }

    let serialized_lines = bincode::encode_to_vec(trie_lines, config::standard()).unwrap();

    let  line_path = Path::new("./serialized_outputs/line_scope/trie-serial.bin");
    let mut new = File::create(line_path).unwrap();
    new.write_all(&serialized_lines).unwrap();

    let mut trie_words = Trie::new();
    for words in dataset_words {
        if words.len() > 255 { continue; }  
        trie_words.store(words.to_string());
    }

    let serialized_words = bincode::encode_to_vec(trie_words, config::standard()).unwrap();

    let  word_path = Path::new("./serialized_outputs/word_scope/trie-serial.bin");
    let mut new = File::create(word_path).unwrap();
    new.write_all(&serialized_words).unwrap();
}
