#![feature(async_closure)]
use sdb::prelude::*;
use serde::{Deserialize, Serialize};
use simplelog::*;

async fn run() -> Result<(), SdbError> {
    // Create client
    let client = SurrealClient::new("ws://test_user:test_pass@127.0.0.1:8000/example/demo")
        .build()?;

    // Run a query
    let books_by_george = sdb::query!( (client, "George") => 
        Vec<BookSchema> = "SELECT * FROM books WHERE <-wrote<-authors.name ?~ $0" 
    );

    // handle errors
    let good_books = books_by_george.expect("Query to succeed. Is the example db running?");

    // List results, or bubble the error
    println!("Books by people named 'George':");
    for s in good_books {
        println!("  {}\t{}", s.title, s.word_count.unwrap_or_default())
    }

    // Spacing for terminal ease of reading
    println!("");

    // Run a transaction (errors are automatically bubbled)
    sdb::trans_act!( ( client ) => {
        $longest = "SELECT * FROM books ORDER word_count DESC";
        longest_title: String = pluck("title", 1) "SELECT * FROM $longest";
        long_count: i32 = count() "$longest";
        stories: Vec<BookSchema> = "SELECT * FROM $longest";
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
