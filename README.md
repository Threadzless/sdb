# Surreal DataBase client

An unofficial client for SurrealDB designed to work with [`yew`](https://yew.rs/) and uses a custom macro to minimize boiler-plate. 

WIP, so don't use in production, but if you try it out please report errors, unhelpful error messages, missing features, documentation, glitches.

# Example
Taken from `sdb/examples/demo.rs`:
```rust,no_run
use sdb::prelude::*;
use serde::{Serialize, Deserialize};

#[tokio::main]
async fn main() -> Result<(), SdbError> {
    let client = SurrealClient::new("ws://test_user:test_pass@127.0.0.1:8000/example/demo")
        .build()?;

    // Run some queries
    sdb::trans_act!( ( client ) => {
        $longest = "SELECT * FROM books ORDER word_count DESC FETCH bleep";

        longest_title: String =
            pluck("title", 1) "SELECT * FROM $longest";

        long_count: i32 = count() "$longest";

        stories: Vec<BookSchema> = "SELECT * FROM $longest";
    });

    // Print results
    println!("");
    println!("Longest books: {:?}\n", longest_title);
    println!("All books (ever) {}:", long_count);
    for s in stories {
        println!("  {}", s.title)
    }

    Ok( () )
}

#[derive(Clone, Serialize, Deserialize, SurrealRecord)]
#[table("books")]
pub struct BookSchema {
    pub id: RecordId,
    pub title: String,
    pub word_count: Option<usize>,
}
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
