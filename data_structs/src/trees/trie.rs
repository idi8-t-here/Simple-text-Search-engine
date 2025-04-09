use bincode::{Encode,Decode};

#[derive(Encode, Decode, Debug)]
pub struct Trie {
    children: [Option<Box<Trie>>; 27],  // 26 letters + space
    is_terminal: bool,
    value: Option<Vec<String>>,
    count:u16,
}

impl Default for Trie {
    fn default() -> Self {
        Self::new()
    }
}

impl Trie {
    pub fn new() -> Self {
        Self {
            children: std::array::from_fn(|_| None),
            is_terminal: false,
            value: None,
            count: 0,
        }
    }

    
    pub fn store(&mut self, key: String) {
        let mut node = self;
        let mut value = Vec::new();

        // Traverse the Trie for each character of the key
        for char in key.chars() {
            let lower = char.to_ascii_lowercase();
            let index = match lower {
                'a'..='z' => (lower as u8 - b'a') as usize,
                ' ' => 26,  // Handle space characters
                _ => continue,  // Skip non-alphabetic characters
            };

            // If the node for this character doesn't exist, create a new one
            if node.children[index].is_none() {
                node.children[index] = Some(Box::new(Trie::new()));
            }

            // Move to the next node in the Trie
            node = node.children[index].as_mut().unwrap();
        }

        // When we've traversed all characters, mark this as a terminal node
        if node.is_terminal {
            // Word already exists, just increment count
            match &mut node.value {
                Some(x) => x.push(key),
                None => panic!("Should be a value already")
            }
            node.count += 1;
        } else {
            // Word does not exist, store it as a new terminal node
            value.push(key);
            node.is_terminal = true;
            node.value = Some(value);
            node.count = 1; // Start the count for this word at 1
        }
    }


    pub fn search(&self, prefix: String) -> Result<Option<Vec<String>>, String> {
        let mut node = self;
        
        // Traverse to the end of the prefix
        for char in prefix.chars() {
            let index = if char == ' ' { 26 } else { (char as u8 - b'a') as usize };
            
            if node.children[index].is_none() {
                return Err(format!("No words with prefix '{}'", prefix));
            }
            node = node.children[index].as_ref().unwrap();
        }
        
        // Collect all words from this node down
        let mut results = Vec::new();
        self.collect_words(node, &mut results);
        
        if results.is_empty() {
            Err("No words found".to_string())
        } else {
            Ok(Some(results))
        }
    }
    
    fn collect_words(&self, node: &Trie, results: &mut Vec<String>) {
        if node.is_terminal {
            if let Some(value) = &node.value {
                for word in value {
                    results.push(word.to_string());
                }
            }
        }
        
        for child in node.children.iter().flatten() {
            self.collect_words(child, results);
        }
    }
}
