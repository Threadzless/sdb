# Surreal DataBase client

An unofficial client for SurrealDB designed to work with [`yew`](https://yew.rs/) and uses a custom macro to minimize boiler-plate. 

WIP, so don't use in production, but if you try it out please report errors, unhelpful error messages, missing features, documentation, glitches.

Features
------------

 - ✅ Execute Queries
 - ✅ Temporary variables (**LET** statements)
 - ✅ Variable loading with sanitization
 - ✅ `wasm32`/`wasm64` target support
 - ✅ Websocket protocol
 - ✅ Authentication (Username, Username + Password)
 - 🚧 Transaction Macro
   - ✅ Serialise query results
   - ✅ Error routing
   - 🚧 Syntax helper
   - 🚧 Query helper methods
   - 🚧 Compile time query validation against db
     - 🚧 Query syntx validity
     - ❌ Result parse checking
   - ❌ Check var presence
   - ❌ Verify Struct's match DB schema
 - ❌ Websocket Event recievers

<br/>

# **Getting Started**
## Run the crate example
For if you just wanna jump right in figure it out as you go. Be sure to install [SurrealDB](https://surrealdb.com/install) locally for the demo to work out of the box.

```bash
# Clone repo
git clone https://github.com/Threadzless/sdb
cd sdb

# start local surreal instance with some demo data
./launch-demo-db.sh

# run demo
cargo run --example mega-demo
```

## Crash Course by Example ##
------------

**Note:** ⚠️ Examples in README.md will start with `sdb::example!{` and end with `}`. Ignore this. It's just for doctests

```rust
async fn crash_course( ) -> Result<(), SdbError> {

    // open a client
    let client = SurrealClient::open("ws://test_user:test_pass@127.0.0.1:8000/example/demo")
        .build()?;

    // run a single query
    let ten_books = sdb::query!( (client) => {
        "SELECT * FROM books LIMIT 10" -> Vec<BookSchema>
    });

    // A more complex query
    let author_search_terms = "George";
    let georges_shortest_book = sdb::query!( (client, author_search_terms) => {
        "SELECT * FROM books WHERE <-wrote<-authors.name ?~ $0 ORDER BY word_count ASC"
            -> Option<BookSchema>
    });

    // Multiple queries bundled together
    let min_word_count = 250_000;
    sdb::trans_act!( ( client, min_word_count ) => {
        // Get just the title of the book with the most words
        longest_title: String = pluck("title", 1) 
            "SELECT * FROM books ORDER word_count DESC";

        // Get the number of books in total
        long_count: i32 = count() "SELECT * FROM books WHERE $longest";

        // Retrieve all of the books
        random_book: Option<BookSchema> = "SELECT * FROM books LIMIT 1";
    });

    // Print results
    println!("Longest books: {:?}", longest_title);
    println!("");
    println!("All books (ever) {}:", long_count);
    for s in stories {
        println!("  {}", s.title)
    }

    Ok( () )
}

//
// Schema Definitions. 
//

#[derive(Deserialize, SurrealRecord)]
#[table("books")]
struct BookSchema {
    pub id: RecordId,
    pub title: String,
    pub word_count: usize,
    pub summary: Option<String>,
    pub author: RecordLink<AuthorSchema>,
}

#[derive(Deserialize, SurrealRecord)]
#[table("authors")]
struct AuthorSchema {
    pub id: RecordId,
    pub name: String,
    pub is_alive: Option<bool>,
}

//
// Stuff to make the doctest work. You can ignore this.
//
use serde::Deserialize;
use sdb::prelude::*;

tokio_test::block_on(async {
    crash_course().await.unwrap()
});
```



## Sugar Queries
The macros have various methods which reformat and wrap queries to make it more clear what the goal of a given query is.

#### `count()`
Returns a the number of results of the inner query
```rust
sdb::example!{
    let short_book_count = sdb::query!( ( client ) => {
        "SELECT * FROM books WHERE word_count < 75_000" -> i32
    });
};
```
