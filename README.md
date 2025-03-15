# Simple text search engine
This is a simple text search engine implemented using regex to find matches and the levenshtien distance to prioritize the results.

## Implementation Details
Although i use a single method for searching the user can search on a "by_Word" or "by_Line" scope, and the regex pattern i decided to use 
for each are 
i) r"\w+" = for the "by_Word" scope <br>
ii) r"(?m)^.*\b{&USER_INPUT_HERE}[.,!?;:']?\b.*$"] = for the "by_Line" scope
