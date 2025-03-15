use std::path::PathBuf;
use std::fs;
use regex::Regex;
use levenshtein::levenshtein;

#[derive(Debug)]
enum SearchType {
    Prefix,
    Suffix,
    Contains
}

#[derive(Debug)]
enum Scope {
    Words,
    Lines,
}

#[derive(Debug)]
pub struct UserInput {
    search_scope: Scope,
    search_term:String,
    search_type:SearchType
}

impl UserInput {
    pub fn inital (scope:Option<u16>, search_term:String, types:Option<u16>) -> Self {
        let search_scope = match scope {
            Some(1) => Ok(Scope::Words),
            Some(2) => Ok(Scope::Lines),
            Some(_) => Err(eprintln!("Enter the correct option")),
            None => Err(eprintln!("ran into unexpected error ")),
        }.expect("Invalid scope input");

        let search_type = match types {
            Some(1) => Ok(SearchType::Prefix),
            Some(2) => Ok(SearchType::Suffix),
            Some(3) => Ok(SearchType::Contains),
            Some(_) => Err(eprintln!("Enter the correct option")),
            None => Err(eprintln!("ran into unexpected error ")),
        }.expect("Invalid type input");
        Self { search_scope, search_term, search_type }
    }
    
    pub fn search (&self) {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("Dataset/output.txt");
        let hay = fs::read_to_string(&path).unwrap();
        let hay = hay.replace("\r", "");
        let mut hashset:Vec<(u16,&str)> = Vec::new();

        let regex_pattern = match self.search_scope {
            Scope::Words => r"\w+".to_string(),
            Scope::Lines => format!(r"(?m)^.*\b{}[.,!?;:']?\b.*$", regex::escape(&self.search_term)),
        };

        let re = Regex::new(&regex_pattern).expect("invalid regex pattern");
        let dataset: Vec<_> = re.find_iter(&hay).map(|mat| mat.as_str()).collect();

        match self.search_type {
            SearchType::Prefix => {
                for value in dataset.iter() {
                    let key = levenshtein(self.search_term.as_str().trim(),value);
                    if value.starts_with(self.search_term.as_str().trim()) {
                        hashset.push((key as u16, value));
                    }
                }
            }
            SearchType::Suffix => {
                for value in dataset.iter() {
                    let cleaned_value = value.trim_end_matches(|c: char| !c.is_alphanumeric()); // Remove trailing punctuation
                    let search_term = self.search_term.as_str().trim();

                    if cleaned_value.ends_with(search_term) {
                        let key = levenshtein(search_term, cleaned_value);
                        hashset.push((key as u16, value));
                    }
                }
            }
            SearchType::Contains => {
                for value in dataset.iter() {
                    let key = levenshtein(self.search_term.as_str().trim(),value);
                    if value.contains(self.search_term.as_str().trim()) {
                        hashset.push((key as u16, value));
                    }
                }
            }
        };

        println!("--- should return results for matches only ---");
        //for i in &hashset {
        //    println!("{:?}", i);
        //}
        hashset.sort_by(|a, b| a.0.cmp(&b.0));
        hashset.truncate(100);
        println!("--- results in priority ---");
        for (i,v) in hashset.iter().enumerate() {
            println!("{} -> {:#?}",i+1, v.1);
        }
    }
}
