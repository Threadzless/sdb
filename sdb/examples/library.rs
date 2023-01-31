#![feature(trace_macros)]

use sdb::prelude::*;
use serde::{Deserialize, Serialize};



#[tokio::main]
async fn main() -> Result<(), SdbError> {
    // Open a blank database
    let client = SurrealClient::open("ws://127.0.0.1:8000/example/library")
        .auth_basic("demo_user", "demo_pass")
        .build()?;

    // Clear out existing data
    sdb::query!( client => {
        "REMOVE TABLE authors";
        "REMOVE TABLE books";
    });


    // Populate author table
    sdb::insert!( client => authors (id, name) => [
        ("philip_p", "Philip Pullman"),
        ("susanna_c", "Susanna Clarke"),
        ("george_rrm", "George R. R. Martin"),
        ("leo_t", "Leo Tolstoy"),
        ("charles_d", "Charles Dickens")
    ])?;

    //
    sdb::query!( client =[  ]=> {
        r#"INSERT INTO books (title, word_count, author) VALUES
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
        "#;
    });

    // Get all books who's title contains the word "house"
    let search = "house";
    let books_with_house_in_the_title = sdb::query!( client =[ search ]=> {
        "SELECT * FROM books WHERE title ~ $search FETCH author" as Vec<Book>
    });
    for book in books_with_house_in_the_title {
        println!("  {}", book.title)
    }

    sdb::query!( client => {
        r#"UPDATE books SET word_count = 249736 WHERE title = "Anna Karenina" TIMEOUT 5s"#;
    });

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