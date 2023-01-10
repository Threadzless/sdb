
/*!
Surreal Data Base client
=========================

A very-not official client for SurrealDb, with macros and maybe compile time testing
one day. idk if i can figure that out.

Macro Tutorial
==============

This tutorial expects you to have familiarity with [SurrealDb](https://surrealdb.com/)
and rust. If you don't, why are you here?

Also, for the example code, assume the following unless stated otherwise:
 - The code is in an `async` scope (See the [Rust Async Book](https://rust-lang.github.io/async-book/01_getting_started/01_chapter.html))
 - The demo database is running (run `./launch-demo-db.sh`)

The code can still work if only the `async` condition is met, but the demo database 
makes trying out examples easy.

## Basics 
The [`sdb`] crate comes with two macros for processing queries:
 - [`query!`], for executing a single query 
 - [`trans_act!`] for execugin multiple queries together

### [`query!`] Example
```rust
# use sdb_base::prelude::*;
# use serde::{Serialize, Deserialize};
# 
# async fn test_main() -> SdbResult<()> {
#    let client = SurrealClient::demo()?;
#
#    let search_terms = "StarTrek";
# 
let trek_books = sdb::query!{ (client, search_terms) => 
    Vec<BookSchema> = "SELECT * FROM books WHERE title ~ $0"
};

println!("There are {} books about StarTrek", trek_books.len());
# Ok( () )
# }
#
# tokio_test::block_on( async {
#     test_main().await.unwrap()
# });
#
# #[derive(Clone, Deserialize)]
# pub struct BookSchema {
#     pub id: RecordId,
#     pub title: String,
#     pub word_count: Option<usize>,
# }
```
These work basically the same, except [`query!`] can only execute a single query and 
return a single result. Attempting to put multiple expressions by seperating them with
semicolons (`;`) will result in compile errors.

The advantage to [`query!`] over [`trans_act!`] is that it currently has a 
way to return a [`std::future::Future`] which resolves the query result. 
See  [Error Handeling](# Error Handeling) for details

### [`trans_act!`] Example
The [`trans_act!`] macro is very similar to the [`query!`] macro, but with the added bonus 
of being able to execute multiple queries and return multiple results.

The down side is that there is not a built in way to manage errors, so you must wrap the 
macro call in an `async` function which return a [`Result<, SdbError>`] for it to
compile.

```rust
# use sdb_base::prelude::*;
# use serde::{Serialize, Deserialize};
# 
# async fn test_main() -> SdbResult<()> {
let client = SurrealClient::demo()?;

let author = "George R R Martin";

sdb::trans_act!( ( client, author ) => {
    $good_books = "SELECT * FROM books WHERE <-wrote<-authors.name ?~ $0";
    good_book_count: usize = count() "$good_books";
    longest_books: Vec<BookSchema> = "SELECT * FROM $good_books ORDER BY word_count DESC LIMIT 5";
});

println!("There are {good_book_count} long books by {author}");
# Ok( () )
# }
#
# tokio_test::block_on( async {
#     test_main().await.unwrap()
# });
#
# #[derive(Clone, Deserialize)]
# pub struct BookSchema {
#     pub id: RecordId,
#     pub title: String,
#     pub word_count: Option<usize>,
# }
```

This example will do the following:
 - Connect to the SurrealDb instance described in `client`,
 - Store the value `50_000` into a transaction variable named `$0`.
 - Run the query `SELECT * FROM books WHERE word_count > $0 LIMIT 5`
 - Parse the reply, and store it in `longest_books` as a [`Vec<StorySchema>`].

The variable `longest_books` will in the same scope as the macro.

### Multiple queries
A single [`trans_act!`] macro can include multiple queries. Each query definition must
be seperated by semicolon (`;`) to compile.

```rust
# use sdb_base::prelude::*;
# use serde::{Serialize, Deserialize};
# 
# async fn test_main() -> SdbResult<()> {
# let client = SurrealClient::demo().unwrap();
sdb::trans_act!( ( client, 50_000 ) => {
    longest_books: Vec<BookSchema> =
        "SELECT * FROM books WHERE word_count > $0 LIMIT 5";
    most_published_authors: Vec<AuthorSchema> =
        "SELECT * FROM authors ORDER BY book_count DESC LIMIT 5";
});
# Ok( () )
# }
#
# tokio_test::block_on( async {
#     test_main().await.unwrap()
# });
#
# #[derive(Clone, Deserialize)]
# pub struct BookSchema {
#     pub id: RecordId,
#     pub title: String,
#     pub word_count: Option<usize>,
# }
#
# #[derive(Clone, Deserialize)]
# pub struct AuthorSchema {
#     pub id: RecordId,
#     pub name: String,
# }
```
When multiple queries are bundled together, any single query experiencing an error will
result in the entire transaction being canceled, and the databases contents will be unaffected.

You cannot combine multiple queries into the same string. They must be seperate strings.
To enforce this, any query containing a semicolon (`;`) will fail to compile. Semicolons in
string can brought into queries using transaction variables without issue.

Query Results
-------------
Each query in a transaction must do one of the following to its results:
 - Ignore them. Query will still execute.
 - Store in a transaction variable
 - Parse and store in a rust variable

```rust
# use sdb_base::prelude::*;
# use serde::{Serialize, Deserialize};
# 
# async fn test_main() -> SdbResult<()> {
#     let client = SurrealClient::demo().unwrap();
    sdb::trans_act!( ( client ) => {
        // Ignore them
        "INFO FOR TABLE books";

        // Stored in a transaction variable. Transaction variables must start with `$`
        $ordered_books = "SELECT * FROM books ORDER BY word_count DESC";

        // Parse and store in a rust variable
        longest_books: Vec<BookSchema> = "SELECT * FROM $ordered_books LIMIT 5";
        //                                              ^^^^^^^^^^^^^^
        //            Previous querie's results are being queried ---|
    });
#     Ok( () )
# }
# tokio_test::block_on( async {
#     test_main().await.unwrap()
# });
# 
# #[derive(Clone, Deserialize)]
# pub struct BookSchema {
#     pub id: RecordId,
#     pub title: String,
#     pub word_count: Option<usize>,
# }
```

### Mutable results.
If you need to mutate a query result, you can add a `mut` modifier before the
variable name, like so:

```rust
# use sdb_base::prelude::*;
# use serde::{Serialize, Deserialize};
# 
# async fn test_main() -> SdbResult<()> {
#     let client = SurrealClient::demo().unwrap();
    sdb::trans_act!( ( client, 50_000 ) => {
        mut long_books: Vec<BookSchema> = "SELECT * FROM books WHERE word_count > $0";
    //  ^^^
    //  |-- `long_books` can be mutated, but won't affect the database contents.
    });
#     Ok( () )
# }
# tokio_test::block_on( async {
#     test_main().await.unwrap()
# });
#
# #[derive(Clone, Deserialize)]
# pub struct BookSchema {
#     pub id: RecordId,
#     pub title: String,
#     pub word_count: Option<usize>,
# }
```


### Types

## Error handeling
For [`trans_act!`], all [`Result`](core::result::Result)s are bubbled (`?`) and this
currently cannot be changed. To catch errors, wrap the macro call in a method 
which returns a [`SdbResult`](sdb_base::prelude::SdbResult)

For [`query!`], you can insert an exclimation mark to disable bubbling 
the errors by default
```rust
# use sdb_base::prelude::*;
# use serde::{Serialize, Deserialize};
# 
# tokio_test::block_on( async {
# let client = SurrealClient::demo().unwrap();
    //        Don't bubble errors ---|
    //                               v
    let longest_books = sdb::query!( ! ( client, 25 ) => 
            Vec<BookSchema> = "SELECT * FROM books LIMIT $0"
        );

    let books = longest_books.await.unwrap();
    for book in books {
        println!("  {}", book.title);
    }
# });
#
# #[derive(Clone, Deserialize)]
# pub struct BookSchema {
#     pub id: RecordId,
#     pub title: String,
#     pub word_count: Option<usize>,
# }
```
For [`trans_act!`], all errors are 


## Transaction variables
SurrealDb Transactions can contain variables, which can be referenced in queries. These
are completely seperate from rust variables and only last for the duration of the transaction
they are a part of.

### Loading Transaction variables
The first part of this macro, before the fat arrow (`=>`), is a list of variables
enclosed in parenthesies.

The first variable, `client` is required, and must be a [`SurrealClient`] representing
the SurrealDb database you are querying. It is not a transaction variable, and is not
accessible inside the queries.

### Varibale Parsing
All variables after `client` are sanitized and included in the transaction. Queries can then
reference their values. By default, variables will be named, in order of apperence, as `$0`,
`$1`, `$2`, and so on. The can be named by following the variable or expression with
`as $new_name`.

Transaction variables must implement [`serde::Serialize`]. This requirement is
already met for primitive types ([`u32`], [`f64`], [`char`], ect.), [`String`]s, `&`[`str`]s,
and a decent number of types in other libraries, which is usually feature gated. See you
librarie's docs for more info

Also, for any type `T` which implement [`Serialize`](serde::Serialize), [`Vec<T>`]s and [`Option<T>`]s
implement it as well.

Also also, [`sdb`] provides some generic types:
 - [`Value`](serde_json::Value), a re-export of `serde_json::Value`, which can contain the results of any
 successful SurrealDb query, but requires more work to extract values.
 - [`AnyRecord`], which is like `Value`, but requires the result to contain an `id` field, which
 all SurrealDb records do.

### Store Transaction Variables


### Limits
There is not hard limit on the number of variables which may be used, but if you're
moving arround multiple gigabytes of data in one transaction, you *might* run into
performance issues, or exceed Surreal's max query size, which will throw an runtime error.

The maximum query size can be changed, I think **TODO:** explain how or link to Surreal Docs


Sql Methods
===========
This macro also also integrates some helper methods for manipulating the sql results
without wiriting a bunch of SurrealQL boilerplate.

For example, by combining the `pluck("name")` and `limit(3)` methods, you can
extract the `name` field from the first 3 records, and store them as a [`Vec<String>`]
```rust
# use sdb_base::prelude::*;
# use serde::{Serialize, Deserialize};
# 
# async fn test_main() -> SdbResult<()> {
# let client = SurrealClient::demo().unwrap();
sdb::trans_act!( (client) => {
    oldest_people_names: [String] =
        pluck("name") limit(5)
        "SELECT * FROM people ORDER BY age DESC";
});
# Ok( () )
# }
# tokio_test::block_on( async {
#     test_main().await.unwrap();
# });
#
# #[derive(Clone, Deserialize)]
# pub struct BookSchema {
#     pub id: RecordId,
#     pub title: String,
#     pub word_count: Option<usize>,
# }
```
These methods work by wrapping the existing query in more queries. the original query
will be `<Q>` from here on.

All arguments passed must be literals. You cannot inject variables this way (yet). Yes I
do know this kinda defeats the purpose.

## Methods
### pluck( `field_name` \[ , `type` \] )
Retrieves a single field from a set of records. If `type` is included, the fields will be
passed through surreal's `type::<type>( )` function
```sql,no_test
SELECT * FROM (SELECT <field_name> FROM ( <Q> ))
```
or
```sql,no_test
SELECT * FROM (SELECT type::<type>( <field_name> ) AS <field_name> FROM ( <Q> ))
```

### limit( `max_results` \[ , `start_offset` \] )
```sql,no_test
SELECT * FROM ( <Q> ) LIMIT <max_results>
```
or
```sql,no_test
SELECT * FROM ( <Q> ) LIMIT <max_results> START <start_offset>
```

### page( `page_size`, `page_number` )
```sql,no_test
SELECT * FROM ( <Q> ) LIMIT <page_size> START (<page_size> * (<page_number> - 1))
```

### shuffle( \[ `limit` \] )
Select a random subset of the results
```sql,no_test
SELECT * FROM ( <Q> ) ORDER BY rand()
```

### one()
Retrieves a maximum of 1 record. Equivilent to `limit(1)`
```sql,no_test
SELECT * FROM ( <Q> ) LIMIT 1
```

### count()
The number of results in the query.
```sql,no_test
SELECT * FROM count(( <Q> ))
```
Common usage:
```rust
# use sdb_base::prelude::*;
# use serde::{Serialize, Deserialize};
# 
# async fn test_main() -> SdbResult<()> {
# let client = SurrealClient::demo().unwrap();
    let search_term = "StarTrek";

    sdb::trans_act!( (client, search_term) => {
        $books = "SELECT * FROM books WHERE title ~ $0";
        count: i32 = count() "$books";
        books: Vec<BookSchema> = limit(20) "$books";
    });

    println!("There are {count} books about StarTrek");
# Ok( () )
# }
# tokio_test::block_on( async {
#     test_main().await.unwrap();
# });
#
# #[derive(Clone, Deserialize)]
# pub struct BookSchema {
#     pub id: RecordId,
#     pub title: String,
#     pub word_count: Option<usize>,
# }
```
*/

pub use sdb_macros::*;

pub use sdb_base::*;

pub mod prelude {
    pub use sdb_base::prelude::*;
    pub use sdb_macros::{
        SurrealRecord
    };
}


// documentation links
#[allow(unused_imports)]
use sdb_base::prelude::*;
#[allow(unused_imports)]
use sdb_base as sdb;