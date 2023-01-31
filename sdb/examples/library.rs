#![feature(trace_macros)]

use sdb::prelude::*;
use serde::{Deserialize, Serialize};



#[tokio::main]
async fn main() -> Result<(), SdbError> {
    // Open a blank database
    let client = SurrealClient::open("ws://127.0.0.1:8000/example/library")
        .auth_basic("demo_user", "demo_pass")
        .build()?;

    // Execute a query, store the result in info
    let info = sdb::query!( client => {
        "INFO FOR DB" as serde_json::Value
    })?;
    println!("Database Info: {:#?}", info);


    // Now multiple statements
    let results = sdb::query!( client => {
        // Only the last statement is parsed, but all are executed
        "USE NS example DB library";
        // `_` is alias for serde_json::Value
        "INFO FOR NS" as _ 
    });
    match results {
        Ok(info) => println!("Namespace Info: {:#?}", info),
        Err(err) => panic!("Something went wrong: {:?}", err),
    }

    
    // Delete existing records. 
    sdb::query!( client => {
        "DELETE books";
    })
    .expect("DELETE query failed");

    // Populate books table
    sdb::insert!( client => books (title, word_count, author) => [
        ("The Subtle Knife", 109120, "authors:philip_p"),
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
        ("Great Expectations", 183349, "authors:charles_d")
    ])?;
    // Insert 

    // Override existing records, if any, and return the inserted
    // record ids
    let _inserted_ids: Vec<RecordId> = 
        sdb::insert!( 
            client => ! authors (id, name) => [
                ("philip_p", "Philip Pullman"),
                ("susanna_c", "Susanna Clarke"),
                ("george_rrm", "George R. R. Martin"),
                ("leo_t", "Leo Tolstoy"),
                ("charles_d", "Charles Dickens")
            ]
            return id
        )?;

    // Start a transaction
    // let mut trans = client.begin();

    // Update some records
    // sdb::query!( trans => {
    //     r#"UPDATE books SET word_count = 249736 WHERE title = "Anna Karenina""#;
    // })?;

    // // Revert the actions of the transaction.
    // trans.cancel();

    // Update again, and this time get the number of records changed
    let records_changed = sdb::query!( client => {
        r#"UPDATE books SET word_count = 249736 WHERE title = "Anna Karenina""#
            .count() as usize
        //  ^^^^^^^^ This is a Query Sugar(TM). 
    })?;

    assert_eq!(records_changed, 1);

    // Query Sugar's act on 


    // // Get all books who's title contains the word "house"
    // // the value of `search_term` is injected into the query at runtime
    // let search_term = "house".to_string();
    // let search_results = sdb::query!(
    //     client =[ search_term ]=> {
    //         "SELECT * FROM books WHERE title ~ $search_term FETCH author" as Vec<Book>
    //     }
    // );
    // for book in search_results {
    //     println!("  {}", book.title)
    // }
    
    

    // let search = "George";
    // sdb::query!( client =[ search ]=> {
    //     // Add up the values of `word_count` in all books
    //     "SELECT * FROM `books` LIMIT 10 WHERE `author`.`name` ~ $search"
    //         .sum("word_count")
    //         => total_word_count: usize;
    //
    //     // Nothing new here
    //     "SELECT * FROM `books` WHERE `author`.`name` ~ $search" => $books_by;
    //
    //     // Query Sugar™s can operate on query vars directly
    //     "SELECT * FROM $books_by" .count()
    //         => author_book_count: usize;
    // });

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