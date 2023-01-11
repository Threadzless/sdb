# Surreal DataBase client

An unofficial client for SurrealDB designed to work with [`yew`](https://yew.rs/) and uses a custom macro to minimize boiler-plate. 

WIP, so don't use in production, but if you try it out please report errors, unhelpful error messages, missing features, documentation, glitches.

### **Quick Start**

For if you want to mess arround with the library right now.

Be sure to install [SurrealDB](https://surrealdb.com/install) locally for the demo to work out of the box.

```bash
# Clone repo
git clone https://github.com/Threadzless/sdb
cd sdb

# start local surreal instance with some demo data
./launch-demo-db.sh

# run demo
cargo run --example demo
```

-----

## **Example**

Taken from `sdb/examples/demo.rs`:

```rust,no_run
#[tokio::main]
async fn main() -> Result<(), SdbError> {
    let client = SurrealClient::new("ws://test_user:test_pass@127.0.0.1:8000/example/demo")
        .build()?;

    // Run a transaction (errors are automatically bubbled)
    sdb::trans_act!( ( client, 150000 ) => {
        // Store results of a query in a transaction variable.
        // Queries that follow can act on these results
        $longest = "SELECT * FROM books WHERE word_count > $0 ORDER word_count DESC ";

        // Get just the title of the first result of `$longest`
        longest_title: String = pluck("title", 1) "$longest";

        // Get the number of books in total
        long_count: i32 = count() "$longest";

        // Retrieve all of the books
        stories: Vec<BookSchema> = "SELECT * FROM books";
    });

    // Print results
    println!("Longest books: {:?}", longest_title);
    println!("");
    println!("All books (ever) {}:", long_count);
    for s in stories {
        println!("  {}", s.title)
    }
    Ok(())
}
```


### **Features**
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
