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
Taken from `sdb/examples/crash-course.rs`
```rust
// Create a SurrealClient
// - Server at 127.0.0.1:8000 connect using websockets 
// - login as demo_user:demo_pass
// - using namespace 'example' and database 'demo'
let client = SurrealClient::open("ws://demo_user:demo_pass@127.0.0.1:8000/example/demo")
    .build()
    .unwrap();


// Run a query
let mut results = client.transaction()
    .push("SELECT * FROM 12")
    .run()
    .await?;
let twelve = results.next_one::<usize>()?;
assert_eq!(twelve, 12);


// Use the query! macro to reduce boilerplate
sdb::query!( client => {
    "SELECT * FROM 12" => twelve_again: usize;
});
assert_eq!(twelve, twelve_again);


// Alternative syntax for single queries
let twelve_yet_again = sdb::query!( client => { "SELECT * FROM 12" as usize });
assert_eq!(twelve_again, twelve_yet_again);
assert_eq!(twelve_yet_again, 12); // just to be sure


// Now run multiple queries
sdb::query!( client => {
    // Update some records, and return zero results
    "UPDATE books SET word_count = 0 WHERE word_count = unset";

    // Parse results as `Vec<Book>` into rust variable `long_books`
    "SELECT * FROM books WHERE word_count > 250000 FETCH author"
        => long_books: Vec<Book>;
});
println!("Here are some long books:");
for book in long_books {
    println!("  - {} by {} has {} words",
        book.title,
        book.author().name,
        book.word_count
    )
}


// Inject variables into the query
let search = "George";
sdb::query!( client =[ search, 5 ]=> {
    // Use variable
    "SELECT * FROM books WHERE author.name ~ $0" => books_by: Vec<Book>;

    // Use variable by name
    "SELECT * FROM books WHERE author.name !~ $search" => books_not_by: Vec<Book>;

    // Store query in a transaction variable then use it later
    "SELECT * FROM books WHERE author.name ~ $search" => $books_by;
    "SELECT * FROM $books_by LIMIT $1" => _five_books_by: Vec<Book>;
});
println!("{search} published {} books", books_by.len());
println!("People not named {search} published {} books", books_not_by.len());


// Use Query Sugar™
let search = "George";
sdb::query!( client =[ search ]=> {
    // Add up the values of `word_count` in all books
    "SELECT * FROM books WHERE author.name ~ $search"
        .sum("word_count") => total_word_count: usize;

    // Nothing new here
    "SELECT * FROM books WHERE author.name ~ $search" => $books_by;

    // Query Sugar™s can operate on query vars directly
    "$books_by" .count() => author_book_count: usize;
});
println!("{search} published {author_book_count} books with a total word count of {total_word_count}");


// use FETCH clause to get nested data
sdb::query!( client => {
    "SELECT * FROM books FETCH author"
        .shuffle() .limit( 5 ) => books_by: Vec<Book>;
});
println!("Here are five books and their authors:" );
for book in books_by {
    println!("  - {} by {}", book.title, book.author().name )
}

//
// Schema definition
//

#[derive(Serialize, Deserialize, SurrealRecord)]
#[table("books")]
pub struct Book {
    pub id: RecordId,
    pub title: String,
    pub word_count: usize,
    // either a RecordId, or an Author. This makes FETCHs way easier
    pub author: RecordLink<Author>,
}

#[derive(Serialize, Deserialize, SurrealRecord)]
#[table("authors")]
pub struct Author {
    pub id: RecordId,
    pub name: String,
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

### **`count( [field] )`**
Returns a the number of results, *OR* the number of results which contain a `field` who's value is truthy
```rust
sdb::query!( client => {
    "SELECT * FROM books WHERE word_count < 75_000"
        .count() => short_count: i32;

    "SELECT (word_count > 75_000) AS long FROM books"
        .count("long") => long_count: i32;
});
```


### **`ids()`**
Retrieves a list of the id's of the result records. 
```rust
sdb::query!( client => {
    "SELECT * FROM books" .ids() => Vec<RecordId>
});
```


### **`limit( max [ , start ] )`**
Adds a `LIMIT max` clause to a **SELECT** query, or `LIMIT max START start`.


### **`one()`**
Adds a `LIMIT 1` clause to a **SELECT** query.


### **`page( size, page )`**
Divides the results into blocks of `size` and returns the `page`th block. Useful for paging.


### **`pluck( field [ , max ] )`**
Gets an array containing the value of a given field from each result, and optionally appends a `LIMIT max` clause.

**Note:** *There are plans to also parse multiple fields into tuples, but this is not implemented*

```rust
sdb::query!( client => {
    "SELECT * FROM books WHERE word_count < 75_000"
        .pluck("title") => Vec<String>
});
```

### **`product( field )`**
Gets gets `field` from every record and calculates the product

```rust
// Get ... a big number? idk when you'd actually use this
sdb::query!( client =[ "George" ]> {
    "SELECT * FROM books WHERE author.name ~ $0"
        .product("word_count") => big_number: usize
});
```

### **`shuffle( [ max ] )`**
Gets a randomized list of results, and optionally set a maximum number to return.

Same as adding a `ORDER BY rand() LIMIT max` clause


### **`sum( field )`**
Gets gets `field` from every record and calculates the sum

```rust
// Get the number of words written by authors named "George"
sdb::query!( client =[ "George" ]> {
    "SELECT * FROM books WHERE author.name ~ $0"
        .sum("word_count") => words_written: usize
});
```