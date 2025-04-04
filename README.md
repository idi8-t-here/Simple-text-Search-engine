## Simple Text Search Engine 

### Description
>The idea behind the simple text search engine, is to create an engine that tokenizes user input(search term) and the dataset(dictionary) 
>then efficently matches the tokens to get the value and gives it a ranking based on the priority.

### Implemetation Details
| List of resources we will use | Why? |
| ------------- | ---|
| [Radix tree wiki](https://en.wikipedia.org/wiki/Radix_tree) | - For PREFIX_SEARCH Implemetation  |
| [Suffix tree wiki](https://en.m.wikipedia.org/wiki/Suffix_tree) | - For SUFFIX_SEARCH Implemetation |
| [Inverted index wiki](https://en.wikipedia.org/wiki/Inverted_index) | - For CONTAINS_SEARCH Implemetation |

| List of crates we will use | Why? |
| ------------- |---|
| [Unicode Segmentation](https://crates.io/crates/unicode-segmentation) | - For helping with search scope i.e Tokenization of words or lines |
| [Levenshtein](https://crates.io/crates/levenshtein)  | - For dictating the method by which we Rank search results |
| [thiserror](https://crates.io/crates/thiserror)  | - For custom error definitions in codebase |
| [bincode](https://crates.io/crates/bincode)  | - For processing dataset into binary  |
| [clap](https://crates.io/crates/clap)  | - For augmenting UI experience |

#### Solving the problem
> 1-  SEARCH_SCOPE  > an enum that has the types of scope the search will be conducted on...[words/lines]
> 2-  SEARCH_TYPE  > an enum that has they types of search we want to conduct...[prefix/contains/suffix]

##### Solving issue 1 (SEARCH_SCOPE)
> In order to solve the issue of how we would be able to tokenize correctly in the scope of words(tokenize characters) and scope of lines(tokenize words)
> we will use [Unicode Segmentation crate](https://crates.io/crates/unicode-segmentation) which will handle this issue for us

##### Solving issue 2 (SEARCH_TYPE)
> In order to handle the different types of Searchs we have we will use the <br/>
> [Radix tree](https://en.wikipedia.org/wiki/Radix_tree) data structure to store our dataset and retrive prefix searches from...<br/>
> [Suffix tree](https://en.m.wikipedia.org/wiki/Suffix_tree) data structure to store our dataset and retrive suffix searches from...<br/>
> [Inverted index](https://en.wikipedia.org/wiki/Inverted_index) data structure to store our dataset and retrive contains searches from...<br/>

#### How it'll Run
1. On build phase with the use of pre-compilation hooks the engine will process(STORE) the dataset(dictionary) into all three types of data structures and serialize it for later use(runtime)
1. Will greet user with a set of options that'll make the user choose what SEARCH_SCOPE and SEARCH_TYPE they want to use
3. After user enters they're search term, the engine will return results in a decending order(based on ranking)
