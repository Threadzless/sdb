#![feature(async_closure)]
use sdb::prelude::*;
use serde::{Deserialize, Serialize};
use simplelog::*;

async fn run() -> Result<(), SdbError> {
    // Create client
    let client = SurrealClient::open("127.0.0.1:8000/example/demo")
        .auth_basic("demo_user", "demo_pass")
        .protocol(Protocol::Socket { secure: false })
        .build()?;

    let search_term = "George";

    // Run a query on `client`
    sdb::query!( client =[ search_term ] => {
        // $0 is the refers to the first var inside the brackets. Vars can be 
        // literals or expressions. if the var is a 
        "SELECT * FROM books WHERE author.name ~ $0"
            => books_by: Vec<BookSchema>;
        
        "SELECT * FROM books WHERE author.name !~ $search_term"
            .count() => books_not_by_count: usize;
    });

    // List results
    println!("There are {books_not_by_count} NOT by people named {search_term}");
    println!("Books by people named {search_term}");
    for s in books_by {
        println!("  {}   ~{} words", s.title, s.word_count.unwrap_or_default())
    }


    let min_word_count = 250000;

    // Run multiple queries together (errors are automatically bubbled)
    sdb::query!( client => {

        { min_word_count } => $mwc;

        // Store results of a query in a transaction variable.
        // Queries that follow can act on these results
        "SELECT * FROM books WHERE word_count > $mwc ORDER word_count DESC" => $longest;

        // Get just the title of the first result of `$longest`,
        // aka, the book with the most words
        "SELECT title FROM $longest LIMIT 1" => longest_title: String;

        // Get the number of books in total with more than 250k words
        "SELECT * FROM $longest" .count() => long_count: i32;

        // Retrieve all of the books
        "SELECT * FROM books FETCH author" => stories: Vec<BookSchema>;
    });

    // Print results
    println!("Longest books: {:?}", longest_title);
    println!("");
    println!("There are {long_count} books with over {min_word_count} words");
    println!("");
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

//
//
//
//
//

fn main() {
    // Logging
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();

    // run async
    let pool = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    pool.block_on(run()).unwrap();
}
