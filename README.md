# Surreal DataBase client

An unofficial client for SurrealDB designed to work with [`yew`](https://yew.rs/) and uses a custom macro to minimize boiler-plate. 

WIP, so don't use in production, but if you try it out please report errors, unhelpful error messages, missing features, documentation, glitches.

## Features
 - ✅ Execute Queries
 - ✅ Transaction variables
 - ✅ Variable injection with sanitization
 - ✅ `wasm32`/`wasm64` target support
 - ✅ Websocket protocol
 - 🚧 Authentication
    - ✅ just username
    - ✅ username + password
    - ❌ authentication token
 - 🚧 `query!` macro
   - ✅ Serialise query results
   - ✅ Check variable presence
   - 🚧 ~ *Query Sugar™* ~
   - 🚧 Syntax checking
     - 🚧 Useful error messages
   - 🚧 Compile time query validation against db
   - ❌ Unpack results into tuple
 - ❌ Websocket Event recievers
 - ❌ Compile time schema validation
 - ❌ Record type schema checking

## Run the crate example
For if you just wanna jump right in figure it out as you go. Be sure to install [SurrealDB](https://surrealdb.com/install) locally for the demo to work out of the box.

```bash
# Clone repo
git clone https://github.com/Threadzless/sdb
cd sdb

# start local surreal instance with some demo data
./launch-demo-db.sh

# run demo
cargo run --example demo
```

# **Crash Course by Example**
Taken from `sdb/examples/mega-demo.rs`
```rust
// Open a new client
let client = SurrealClient::open("ws://127.0.0.1:8000/example/demo")
    .auth_basic("test_user", "test_pass")
    .build()?;


// Execute a basic query
sdb::query!( client => {
    "SELECT * FROM 12" => twelve: isize
});
if twelve == 12 {
    println!("1) Twelve does in fact equal 12");
}

// Alternative syntax for single values
let twelve_two = sdb::query!( client => { "SELECT * FROM 12" as isize });
if twelve_two == 12 {
    println!("   Twelve continues to equal 12");
}


// Now multiple queries. Results are parsed with `serde::Deserialize`
sdb::query!( client => {
    // Run this query, ignore the results.
    "UPDATE books SET word_count = 0 WHERE word_count < 0";

    // Zero or more results
    "SELECT * FROM books ORDER rand()" => all_books: Vec<BookSchema>;

    // Just one. Getting zero results throws an error
    "SELECT * FROM books ORDER rand()" => a_book: BookSchema;

    // Returns the first results, or None if there are 0 results.
    "SELECT * FROM books ORDER rand()" => another_book: Option<BookSchema>;
});

println!("2) There are {} books in total", all_books.len());
println!("   There is, allegedly, a book called {}", a_book.title);
if let Some( ab ) = another_book {
    println!("   And maybe a book called {}", ab.title);
}


// Now store results in a mutable field
sdb::query!( client => {
    "SELECT * FROM books ORDER rand() LIMIT 5" => mut five_books: Vec<BookSchema> 
});

println!("3) Five books:");
while let Some( book ) = five_books.pop() {
    println!("     - {}", book.title)
}


// Fetch nested data
sdb::query!( client => {
    "SELECT * FROM books LIMIT 3 FETCH author" => some_books: Vec<BookSchema>;
});
println!("4) Three books and their authors:");
for book in some_books {
    println!("     - '{}' by {}", book.title, book.author().name)
}


// Inject variables
let search_term = "George";
sdb::query!( client =[ search_term, 6 ]=> {
    // $0 = `search_term`,  $1 = 6, and so on
    "SELECT * FROM books WHERE author.name ~ $0 LIMIT $1"
        => by_george: Vec<BookSchema>;

    // passed vars in =[ .. ]=> can also be refered to by name
    "SELECT * FROM books WHERE author.name ~ $search_term LIMIT $1"
        => not_by_george: Vec<BookSchema>;
});
println!("5) Books by '{}'", search_term);
for book in by_george {
    println!("     - {}", book.title)
}    
println!("   Books they didn't write:");
for book in not_by_george {
    println!("     - {}", book.title)
}

// Use Query Sugar™
// See README or macro docs for a list of sugars
sdb::query!( client => {
    "SELECT * FROM books" .count() => book_count: i32;

    // This does the same as the above,
    "SELECT * FROM count(( SELECT * FROM books ))";
});
println!("6) There are {book_count} books in the archive");


// Break up queries into multiple lines.
sdb::query!( client => {
    // Store first query result in a transaction variable
    "SELECT * FROM books ORDER BY word_count DESC" => $longest;

    // Run a query on the results
    "SELECT * FROM $longest LIMIT 3" => mut longest_books: Vec<BookSchema>
});
let longest = longest_books.remove(0);
println!("7) The longest book is called '{}'", longest.title);


// Run multiple queries together.
let long_word_count = 250000;
sdb::query!( client =[ long_word_count ]=> {
    // Store results of a query in a transaction variable.
    // Queries that follow can act on these results
    "SELECT * FROM books WHERE word_count > $0 ORDER word_count DESC" => $longest;

    // Get just the title of the first result of `$longest`, aka, the book
    // with the most words, and store it in
    "$longest" .pluck("title", 1) => longest_title: String;

    // Get the number of books in total with more than 250k words
    "SELECT * FROM $longest" .count() => long_count: i32;

    // Retrieve 10 books, with nested data
    "SELECT * FROM books FETCH author" .limit(10) => stories: Vec<BookSchema>;
});

println!("8) Longest books: '{longest_title}'");
println!("   There are {long_count} books with over {long_word_count} words");
println!("   Several books:");
for s in stories {
    println!("    - '{}' by {}", s.title, s.author().name)
}
```

# Schema Structs.
Because SurrealDB can restructure data pretty significantly, making the corresponding structs for it could get complicated and tedius. To minimise that, there are 3 helper types:
 - `SurrealRecord` - a trait with a derive macro which represents any struct that's a surreal record. 
 - `RecordId` - what it says on the tin
 - `RecordLink< T >` - an enum which can be either a `RecordId`, or a `SurrealRecord`. This makes using **FETCH** clauses way easier.
 
 ```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, SurrealRecord)]
#[table("books")]
pub struct BookSchema {
    // All record structs must have an id field 
    pub id: RecordId,

    // Required field. Parsing will fail if it's missing :(
    pub title: String,
    
    // Optional field. Pwrsing still works if missing :)
    pub word_count: Option<usize>,

    // A RecordLink is either a RecordId, or the contents of another
    // record. This means you can use FETCH clauses without headaches
    pub author: RecordLink<AuthorSchema>,
}

#[derive(Serialize, Deserialize, SurrealRecord)]
#[table("authors")]
pub struct AuthorSchema {
    pub id: RecordId,
    pub name: String,
}
```
And now a usage example:
```rust
sdb::query!( client => {
    "SELECT * FROM books FETCH author" => some_book: BookSchema,
});

println!("{} was written by {}", some_book.title, author.record().name)
```

# `query!` Macros Explained
Boilerplate is tedious, so `sdb` has a macro for writing queries. In addition to reducing tedium, it performs some syntax checks at compile time, like making sure your parenthesies are matched and that your clauses are in the correct order.

## Transaction Variables
In the option arrow block, any variable or expression after the `client` will be parsed and
made available to the query. They will be named `$0`, `$1`, `$2`, and so on. Passed variables can
also be referenced by name.

```rust
sdb::query!( client => {
    let long_word_count = 225_000;
    
    sdb::query!( client =[ long_word_count ]=> {
        //                 └─────────────┴──────┬┐ 
        "SELECT * FROM books WHERE word_count > $0 "
            .count() => number_of_long_books: i32;

        // Var can be accessed by name          ┌──────────────┐
        "SELECT * FROM books WHERE word_count > $long_word_count"
            .count() => number_of_long_books: i32;

        // Expressions can be inserted as named variables
        { long_word_count / 2 } => $short;
        
        "SELECT * FROM books WHERE word_count < $short"
            .count() => number_of_short_books: i32;
    });
});
```
All transaction variables must have a dollar sign (`$`) prefix

# ~ *Query Sugar™* ~
The `query!` macro has various methods which reformat and wrap queries to make it more clear what the goal of a given query is.

### `count( [field] )`
Returns a the number of results, *OR* the number of results which contain a `field` who's value is truthy
```rust
sdb::query!( client => {
    "SELECT * FROM books WHERE word_count < 75_000"
        .count() => short_count: i32;

    "SELECT (word_count > 75_000) AS long FROM books"
        .count("long") => long_count: i32;
});
```


### `ids()`
Retrieves a list of the id's of the result records. 
```rust
sdb::query!( client => {
    "SELECT * FROM books" .ids() => Vec<RecordId>
});
```


### `limit( max [ , start ] )`
Adds a `LIMIT max` clause to a **SELECT** query, or `LIMIT max START start`.


### `one()`
Adds a `LIMIT 1` clause to a **SELECT** query.


### `page( size, page )`
Divides the results into blocks of `size` and returns the `page`th block. Useful for paging.


### `pluck( field [ , max ] )`
Gets an array containing the value of a given field from each result, and optionally appends a `LIMIT max` clause.

**Note:** *There are plans to also parse multiple fields into tuples, but this is not implemented*

```rust
sdb::query!( client => {
    "SELECT * FROM books WHERE word_count < 75_000"
        .pluck("title") => Vec<String>
});
```


## `shuffle( [ max ] )`
Gets a randomized list of results, and optionally set a maximum number to return.

Same as adding a `ORDER BY rand()` clause