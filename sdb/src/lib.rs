
/*!
Surreal Data Base client
=========================

A very-not official client for SurrealDb, with macros and maybe compile time testing
one day. idk if i can figure that out.

[`trans_act!`] Macro Tutorial
==============

This tutorial expects you to have familiarity with [SurrealDb](https://surrealdb.com/)
and rust. If you don't, why are you here?

Also, for the example code, assume the following unless stated otherwise:
- An instance of SurrealDb is running on `localhost`
- The code is in an `async` scope (See the [Rust Async Book](https://rust-lang.github.io/async-book/01_getting_started/01_chapter.html))
- Surreal's strict mode is off
- The Surreal instance doesn't require login to execute any kind of query (never do this in a production enviroment)
- The variable `client` is in scope and is created like this:
```rust,no_run
let client = sdb::SurrealClient::new( "127.0.0.1:8000/test/test" ).build();
```

The code can still work if only the `async` condition is met, but the extra code
is needed to do this, and that would distract from the parts relevant to most of this tutorial.

## [`trans_act!`] Macro Basics
Consider this code:
```rust,no_run
sdb::trans_act!( ( client, 50_000 ) => {
    longest_books: Vec<StorySchema> =
        "SELECT * FROM books WHERE word_count > $0 LIMIT 5";
});
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
```rust,no_run
sdb::trans_act!( ( client, 50_000 ) => {
    longest_books: Vec<StorySchema> =
        "SELECT * FROM books WHERE word_count > $0 LIMIT 5";
    most_published_authors: Vec<AuthorSchema> =
        "SELECT *, count(->published) AS book_count FROM authors ORDER BY book_count DESC LIMIT 5";
});
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

```rust,no_run
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
```

### Mutable results.
If you need to mutate a query result, you can add a `mut` modifier before the
variable name, like so:

```rust,no_run
sdb::trans_act!( ( client, 50_000 ) => {
    mut long_books: Vec<BookSchema> = "SELECT * FROM books WHERE word_count > $0";
//  ^^^
//  |-- `long_books` can be mutated, but won't affect the database contents.
});
```


### Types

## Error handeling
By default, all errors are bubbled to the parent method. You can add
a `!` to unwrap() instead.
```rust,no_run
//          Unwrap errors - - - |
//                              v
sdb::query!( ( client, 25 ) ! => {
    longest_books: Vec<StorySchema> = "SELECT * FROM books LIMIT $0";
});
```



## Transaction variables
SurrealDb Transactions can contain variables, which can be referenced in queries. These
are completely seperate from rust variables and only last for the duration of the transaction
they are a part of.

### Loading Transaction variables
The first part of this macro, before the fat arrow (`=>`), is a list of variables
enclosed in parenthesies.

The first variable, `client` is required, and must be a [`sdb::SdbClient`] representing
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

Also, for any type `T` which implement [`serde::Serialize`], [`Vec<T>`]s and [`Option<T>`]s
implement it as well.

Also also, [`sdb`] provides some generic types:
 - [`Value`], a re-export of `serde_json::Value`, which can contain the results of any
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
sdb::trans_act!( (client) => {
    oldest_people_names: [String] =
        pluck("name") limit(5)
        "SELECT * FROM people ORDER BY age DESC";
});
```
These methods work by wrapping the existing query in more queries. the original query
will be `<Q>` from here on.

All arguments passed must be literals. You cannot inject variables this way (yet). Yes I
do know this kinda defeats the purpose.

## Methods
### pluck( `field_name` [[ , `type` ]] )
Retrieves a single field from a set of records. If `type` is included, the fields will be
passed through surreal's `type::<type>( )` function
```
SELECT * FROM (SELECT <field_name> FROM ( <Q> ))
```
or
```
SELECT * FROM (SELECT type::<type>( <field_name> ) AS <field_name> FROM ( <Q> ))
```

### limit( `max_results` [[ , `start_offset` ]] )
```
SELECT * FROM ( <Q> ) LIMIT <max_results>
```
or
```
SELECT * FROM ( <Q> ) LIMIT <max_results> START <start_offset>
```

### page( `page_size`, `page_number` )
```
SELECT * FROM ( <Q> ) LIMIT <page_size> START (<page_size> * (<page_number> - 1))
```

### shuffle( [[ `limit` ]] )
Select a random subset of the results
```
SELECT * FROM ( <Q> ) ORDER BY rand()
```

### one()
Retrieves a maximum of 1 record. Equivilent to `limit(1)`
```
SELECT * FROM ( <Q> ) LIMIT 1
```

### count()
The number of results in the query.
```
SELECT * FROM count(( <Q> ))
```
Common usage:
```rust,no_run
sdb::trans_act( (client, search_term) => {
    $books = "SELECT * FROM books WHERE title ~ $0";
    count: i32 = count() "$books";
    books: Vec<BookSchema> = limit(20) "$books";
})
```
*/

pub use sdb_macros::*;

pub use sdb_base::*;