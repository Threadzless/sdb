# Surreal DataBase client

An unofficial client for SurrealDB designed to work with [`yew`](https://yew.rs/) and uses a custom macro to minimize boiler-plate. 

WIP, so don't use in production, but if you try it out please report errors, unhelpful error messages, missing features, documentation, glitches.

## Features
 - ✅ Execute Queries
 - ✅ Transaction variables
 - ✅ Variable injection with sanitization
 - ✅ `wasm32`/`wasm64` target support
 - ✅ Websocket protocol
 - ✅ Authentication
 - 🚧 Macros!
 - ❌ Websocket Event recievers
 - ❌ Compile time schema validation

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
sdb::doctest!{
    // Create a SurrealDB Client
    let url = "ws://demo_user:demo_pass@127.0.0.1:8000/example/demo";
    let client = SurrealClient::open(url)
        .build()
        .unwrap();


    // Using the client without a macro (cringe)
    let mut results = client.transaction()
        .push("SELECT * FROM [ 12, 15, 3 ]")
        .run()
        .await?;
    let array = results.next_vec::<usize>()?;
    assert_eq!(array[0], 12);
    assert_eq!(array[1], 15);
    assert_eq!(array[2], 3);


    // Now use the the query! macro (pog)
    let numbers = sdb::query!( client => "SELECT * FROM [ 12, 15, 3 ]" as Vec<usize> )
        .unwrap();
    assert_eq!(numbers[0], 12);
    assert_eq!(numbers[1], 15);
    assert_eq!(numbers[2], 3);


    // Use an insert! macro
    // Values can be literals or variables
    let title_1 = "The Subtle Knife";
    sdb::insert!( client => books (title, word_count, author) => [
        (title_1, 109120, "authors:philip_p"),
        ("The Amber Spyglass", 168640, "authors:philip_p"),
        ("The Golden Compass", 117683, "authors:philip_p"),
        ("Jonathan Strange & Mr Norrell", 308931, "authors:susanna_c"),
        ("A Clash of Kings", 326000, "authors:george_rrm"),
        ("A Storm of Swords", 424000, "authors:george_rrm"),
        ("A Game of Thrones", 298000, "authors:george_rrm"),
        ("A Feast for Crows", 300000, "authors:george_rrm"),
        ("Anna Karenina", 0, "authors:leo_t"),
        ("War and Peace", 561304, "authors:leo_t"),
        ("Bleak House", 360947, "authors:charles_d"),
        ("Great Expectations", 183349, "authors:charles_d"),
    ])?;


    // Override existing records, if any, and return the inserted ids
    let _inserted_ids: Vec<RecordId> = 
        sdb::insert!( 
            //        v--- override records with the same id, if there are any
            client => ! authors (id, name) => [
                ("philip_p", "Philip Pullman"),
                ("susanna_c", "Susanna Clarke"),
                ("george_rrm", "George R. R. Martin"),
                ("leo_t", "Leo Tolstoy"),
                ("charles_d", "Charles Dickens")
            ]
            return id
        )?;


    // Update a record with an incorrect value
    sdb::query!( client =>
        // Macro works properly with rust's raw strings.
        r#"UPDATE books SET word_count = 249736 WHERE title = "Anna Karenina" RETURN NONE"# as _
    )?;


    // Now run multiple queries
    // This is a different macro with a slightly different syntax
    // It must be run in an async function which returns SdbResult<T>
    sdb::queries!( client => {
        // Update some records, the server reply isn't stored.
        "UPDATE books SET word_count = 249736 WHERE word_count = unset";

        // Parse results as `Vec<Book>` and store in rust variable `long_books`
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


    // Inject values / variables into the query
    // also get parse multiple queries 
    let search = "George";
    sdb::queries!( client =[ search, 5 ]=> {
        // Use variable
        "SELECT * FROM books WHERE author.name ~ $0" => books_by: Vec<Book>;

        // Use variable by name
        "SELECT * FROM books WHERE author.name !~ $search LIMIT $1"
            => books_not_by: Vec<Book>;

        // Split a query into multiple lines for readability
        "SELECT * FROM books WHERE author.name ~ $search" => $books_by;

        "SELECT * FROM $books_by LIMIT $1" => _five_books_by: Option<Book>;
    });
    println!("{search} published {} books", books_by.len());
    println!("People not named {search} published {} books", books_not_by.len());


    // Use Query Sugar™
    let search = "George";
    sdb::queries!( client =[ search ]=> {
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
    sdb::queries!( client => {
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
}
```


# Schema Structs.
Because SurrealDB can restructure data pretty significantly, making the corresponding structs for it could get complicated and tedius. To minimise that, there are 3 helper types:
- `SurrealRecord` - a trait with a derive macro which represents any struct that's a surreal record. 
- `RecordId` - what it says on the tin
- `RecordLink< T >` - an enum which can be either a `RecordId`, or a `SurrealRecord`. This makes using **FETCH** clauses way easier.
    
```rust
use serde::{Serialize, Deserialize};
use sdb::prelude::*;

//                               Derive Macro
//                               vvvvvvvvvvvvv
#[derive(Serialize, Deserialize, SurrealRecord)]
#[table("books")]
pub struct BookSchema {
    // All record structs must have an id field 
    pub id: RecordId,

    // Required field. Parsing will fail if it's missing :(
    pub title: String,
    
    // Optional field. Parsing still works if missing :)
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
sdb::doctest!{ client => {
    
    let some_book = sdb::query!( client => 
        "SELECT * FROM books FETCH author" as Book
    )?;

    println!("{} was written by {}", some_book.title, some_book.author().name)
}};
```

# `query!` Macros Explained
Boilerplate is tedious, so `sdb` has a macro for writing queries. In addition to reducing tedium, it performs some syntax checks at compile time, like making sure your parenthesies are matched and that your clauses are in the correct order.

## Transaction Variables
In the option arrow block, any variable or expression after the `client` will be parsed and
made available to the query. They will be named `$0`, `$1`, `$2`, and so on. Passed variables can
also be referenced by name.

```rust
sdb::doctest!(client=>{
    
    let long_word_count = 225_000;
    let max = 5;
    sdb::queries!( client =[ long_word_count, max ]=> {
        //                 └─────────────┴──────┬┐ 
        "SELECT * FROM books WHERE word_count > $0 "
            .count() => number_of_long_books: i32;

        // vars can be accessed by name         ┌──────────────┐       ┌──┐
        "SELECT * FROM books WHERE word_count > $long_word_count LIMIT $max"
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
sdb::doctest!(client=>{

    sdb::queries!( client => {
        "SELECT * FROM books WHERE word_count < 75_000"
            .count() => short_count: i32;

        "SELECT (word_count > 75_000) AS long FROM books"
            .count("long") => long_count: i32;
    });

});
```


### **`ids()`**
Retrieves a list of the id's of the result records.
```rust
sdb::doctest!(client=>{

    let book_ids = sdb::query!( client =>
        "SELECT * FROM books" .ids() as Vec<RecordId>
    )?;

});
```


### **`limit( max [ , start ] )`**
Adds a `LIMIT max` clause to a **SELECT** query, or `LIMIT max START start`.


### **`one()`**
Adds a `LIMIT 1` clause to a **SELECT** query.


### **`page( size, page )`**
Divides the results into blocks of `size` and returns the `page`th block. Useful for paging.


### **`pluck( field )`**
Gets a single field from each record as an array of that field, rather than as an
array of objects each with just that field.

**Note:** *There are plans to also parse multiple fields into tuples, but this is not implemented*

```rust
sdb::doctest!(client=>{

    let short_books = sdb::query!( client => 
        "SELECT * FROM books WHERE word_count < 75_000"
            .pluck("title") as Vec<String>
    );

});
```

### **`product( field )`**
Gets gets `field` from every record and calculates the product

### **`shuffle( [ max ] )`**
Gets a randomized list of results, and optionally set a maximum number to return.

Same as adding a `ORDER BY rand() LIMIT max` clause


### **`sum( field )`**
Gets gets `field` from every record and calculates the sum

```rust
sdb::doctest!(client=>{

    // Get the number of words written by authors named "George"
    let words_written = sdb::query!( client =[ "George" ]=> 
        "SELECT * FROM books WHERE author.name ~ $0"
            .sum("word_count") as usize
    );

});
```