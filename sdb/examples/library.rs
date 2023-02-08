#![feature(trace_macros)]

use sdb::{prelude::*};
use serde::{Deserialize, Serialize};



#[tokio::main]
async fn main() -> Result<(), SdbError> {
    // Open a blank database
    let client = SurrealClient::open("ws://127.0.0.1:8000/example/library")
        .auth_basic("demo_user", "demo_pass")
        .build()?;

    // Execute a query, store the result in info
    let info = sdb::query!( client => "INFO FOR DB" as serde_json::Value )?;
    println!("Database Info: {:#?}", info);


    // Now multiple statements
    sdb::queries!( client => {
        // Only the last statement is parsed, but all are executed
        "USE NS example DB library";
        // `_` is alias for serde_json::Value
        "INFO FOR NS" => info: _ 
    });

    println!("Namespace Info: {:#?}", info);

    
    // Delete existing records. 
    // as () to not parse the result, just return a StatementResult
    let result = sdb::query!(
        // don't parse results -----|
        //                          vv
        client => "DELETE books" as _ 
    );
    result.expect("DELETE query failed");

    // Populate books table using the insert! macro
    // Values can be literals or variables
    let title_1 = "The Subtle Knife";
    sdb::insert!{ client => 
        books (title, word_count, author) => [
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
        ]
    }?;


    // Override existing records, if any, and return the inserted ids
    let _inserted_ids: SdbResult<Vec<RecordId>> = sdb::insert!( 
        //        v--- override records with the same id, if there are any
        client => ! authors (id, name) => [
            ("philip_p", "Philip Pullman"),
            ("susanna_c", "Susanna Clarke"),
            ("george_rrm", "George R. R. Martin"),
            ("leo_t", "Leo Tolstoy"),
            ("charles_d", "Charles Dickens")
        ]
        return id
    );

    println!("{:#?}", _inserted_ids);


    // Update a record with an incorrect value
    sdb::query!( client =>
        // Macro works properly with rust's raw strings.
        r#"UPDATE books SET word_count = 249736 WHERE title = "Anna Karenina" RETURN NONE"# as _
    )?;


    // Update again, and this time get the number of records changed
    let records_changed = sdb::query!( client => 
        // Macro works properly with raw and escaped strings.
        r#"UPDATE books SET word_count = 249736 WHERE title = "Anna Karenina""#
            .count() as usize
        //  ^^^^^^^^ This is a Query Sugar(TM). See README for a list of sugars
    )?;
    assert_eq!(records_changed, 1);


    let search_term = "house".to_string();
    // use a variable in the query
    let search_results = sdb::query!( client =[ search_term ]=> 
        "SELECT * FROM books WHERE title ~ $search_term FETCH author" as Vec<Book>
    )?;
    // Any type which implements serde::Serialize can be injected, and 
    for book in search_results {
        println!("  {}", book.title)
    }
    
    
    // multiple expressions in a single pass
    // must be run in a function that returns sdb::SdbResult<T>
    let search = "George";
    sdb::queries!( client =[ search ]=> {
        // Add up the values of `word_count` in all books
        "SELECT * FROM `books` WHERE `author`.`name` ~ $search LIMIT 10 "
            .sum("word_count")
            => _total_word_count: usize;
    
        // Nothing new here
        "SELECT * FROM `books` WHERE `author`.`name` ~ $search" => $books_by;
    
        // Query Sugar™s can operate on query vars directly
        "$books_by" .count()
            => _author_book_count: usize;
    });

    Ok( () )
}

//
// Schema Definition
//

#[derive(Serialize, Deserialize, SurrealRecord)]
#[table("books")]
pub struct Book {
    pub id: RecordId,
    pub title: String,
    pub word_count: Option<usize>,
    pub author: RecordLink<Author>,
}

#[derive(Serialize, Deserialize, SurrealRecord)]
#[table("authors")]
pub struct Author {
    pub id: RecordId,
    pub name: String,
}