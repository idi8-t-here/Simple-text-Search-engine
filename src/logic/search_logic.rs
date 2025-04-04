use std::path::PathBuf;
use std::fs;
use regex::bytes::Regex;
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
        let mut hashset:Vec<(u16,String)> = Vec::new();

        let regex_pattern = match self.search_scope {
            Scope::Words => Regex::new(r"\w+"),
            Scope::Lines => Regex::new(&format!(r"(?m)^.*\b{}[.,!?;:']?\b.*$", regex::escape(&self.search_term))),
        };
        
         let dataset: Vec<_> = match regex_pattern {
            Ok(regex_pattern) => regex_pattern.find_iter(hay.as_bytes()).filter(|mat| mat.as_bytes().len() <= match self.search_scope { Scope::Words => 1020, Scope::Lines => 131072}).map(|mat| mat.as_bytes()).collect(),
            Err(error) => {return eprintln!("Regex compilation failed: {}", error); }
        };

        match self.search_type {
            SearchType::Prefix => {
                for v in dataset.iter() {
                    let value = String::from_utf8_lossy(v).into_owned(); // Convert to owned String
                    let search_term = self.search_term.as_str().trim();
                    if value.starts_with(search_term) {
                        let key = levenshtein(search_term, &value);
                        hashset.push((key as u16, value));
                    }
                }
            }
            SearchType::Suffix => {
                for v in dataset.iter() {
                    let value = String::from_utf8_lossy(v).into_owned(); // Convert to owned String
                    let cleaned_value = value.trim_end_matches(|c: char| !c.is_alphanumeric()).to_string(); // Remove trailing punctuation
                    let search_term = self.search_term.as_str().trim();

                    if cleaned_value.ends_with(search_term) {
                        let key = levenshtein(search_term, &cleaned_value);
                        hashset.push((key as u16, value));
                    }
                }
            }
            SearchType::Contains => {
                for v in dataset.iter() {
                    let value = String::from_utf8_lossy(v).into_owned(); // Convert to owned String
                    let search_term = self.search_term.as_str().trim();
                    if value != search_term && value.contains(search_term) && !value.starts_with(search_term) && !value.ends_with(search_term) {
                        let key = levenshtein(search_term, &value);
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
