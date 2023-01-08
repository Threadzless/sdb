# Surreal DataBase client

An unofficial client for SurrealDB designed to work with [`yew`](https://yew.rs/) and uses a custom macro to minimize boiler-plate. 

WIP, so don't use in production, but if you try it out please report errors, unhelpful error messages, missing features, documentation, glitches.

# Example

```rust,no_run
use sdb::*;

let client = SurrealClient::new("127.0.0.1:8000/example/demo").build().unwrap();

sdb::trans_act!( ( client, 55_000 ) => {
    $ordered_books = "SELECT * FROM books ORDER BY word_count DESC";

    long_books: Vec<BookSchema> =
        "SELECT * FROM $ordered_books WHERE word_count > $0";

    longest_book_chapter: Vec<ChapterSchema> =
        "SELECT * FROM $ordered_books->parts->chapters ORDER BY index";
})
```

### Features
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
     - ❌ Result parse successfull
   - ❌ Check var presence
 - ❌ Websocket Event recievers
