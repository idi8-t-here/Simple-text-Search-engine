use std::time::Instant;
use compile::{ process_data, Trees, Scope};

fn main() {
    let now = Instant::now();
    process_data(Trees::Trie, Scope::Line);
    process_data(Trees::Trie, Scope::Word);

    process_data(Trees::Suffix, Scope::Line);
    process_data(Trees::Suffix, Scope::Word);

    process_data(Trees::NGramIndex, Scope::Line);
    process_data(Trees::NGramIndex, Scope::Word);
    
    let time_taken = now.elapsed().as_secs_f32();
    eprintln!("Time taken to process document - {}", time_taken);
}
