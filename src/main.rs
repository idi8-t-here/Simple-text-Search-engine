use std::io;

mod logic;
use logic::search_logic::UserInput;

fn main() {
    println!("Please enter the search scope");
    println!("->1:for Words \n->2:for Lines");
    let mut input_scope = String::new();
    io::stdin()
        .read_line(&mut input_scope)
        .expect("couldn't read user input");
    println!("Please enter the search type");
    println!("->1:for Prefix search\n->2:for Suffix search\n->3:for Contains search");
    let mut input_type = String::new();
    io::stdin()
        .read_line(&mut input_type)
        .expect("couldn't read user input");

    println!("What term do you want to search for");
    let mut input_term = String::new();
    io::stdin()
        .read_line(&mut input_term)
        .expect("couldn't read user input");
    let scope:u16 = input_scope.trim().parse().unwrap();
    let types:u16 = input_type.trim().parse().unwrap();

    let user_experience = UserInput::inital(Some(scope), input_term.trim().to_string(), Some(types));
    dbg!("{:?}",&user_experience);
    user_experience.search()
}


