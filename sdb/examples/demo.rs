#![feature(async_closure, trace_macros)]
use sdb::prelude::*;
use serde::{Deserialize, Serialize};


#[tokio::main]
async fn main() -> Result<(), SdbError> {
    // Create client
    let client = SurrealClient::open("127.0.0.1:8000/example/demo")
        .auth_basic("demo_user", "demo_pass")
        .protocol(Protocol::Socket { secure: false })
        .build()?;

    let search_term = "George";

    // Run a query on `client`
    sdb::queries!( client =[ search_term ] => {
        // $0  refers to the first var inside the brackets. Vars can be 
        // literals or expressions. if the var is a 
        "SELECT * FROM books WHERE author.name ∋ search_term"
            => books_by: Vec<BookSchema>;
        
        "SELECT * FROM books WHERE author.name !~ $search_term"
            .count() => books_not_by: usize;
    });

    // List results
    println!("There are {books_not_by} NOT by people named {search_term}");
    println!("Books by people named {search_term}");
    for s in books_by {
        println!("  {}   ~{} words", s.title, s.word_count.unwrap_or_default())
    }


    let min_word_count = 249_500;

    // Run multiple queries together (errors are automatically bubbled)
    // notice that this macro is `queries` and not `query`
    // the syntax is slightly different, and query must be run in a 
    // method or closure which returns a [`SdbResult<T>`]
    sdb::queries!( client => {

        // $mwc now equals 250_000
        { min_word_count + 500 } => $mwc;

        // Store results of a query in a transaction variable.
        // Queries that follow can act on these results
        "SELECT * FROM books WHERE word_count > $mwc ORDER BY word_count DESC" => $longest;

        // Get just the title of the first result of `$longest`,
        // aka, the book with the most words
        "SELECT title FROM $longest ORDER BY lerp DESC" => longest_title: String;

        // Get the number of books in total with more than 250k words
        "SELECT * FROM $longest" .count() => long_count: i32;

        // Retrieve all of the books
        "SELECT * FROM books FETCH author" => stories: Vec<BookSchema>;
    });

    // Print results
    println!("Longest books: {longest_title:?}");
    println!();
    println!("There are {long_count} books with over {min_word_count} words");
    println!();
    println!("All books (ever):");
    for s in stories {
        println!("  {}", s.title)
    }

    Ok(())
}

#[derive(Clone, Serialize, Deserialize, SurrealRecord)]
#[table("books")]
pub struct BookSchema {
    pub id: RecordId,
    pub title: String,
    pub word_count: Option<usize>,
}