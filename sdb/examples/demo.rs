#![feature(async_closure)]
use sdb::prelude::*;
use serde::{Deserialize, Serialize};
use simplelog::*;

async fn run() -> Result<(), SdbError> {
    // Create client
    let client = SurrealClient::demo().unwrap();

    // Run a query on `client`
    let books_by_george = sdb::query!( (client, "George") =>
        // $0 is the refers to the first var aside from client. Vars can be either
        // literals or expressions, and will be named in order of occurance
        Vec<BookSchema> = "SELECT * FROM books WHERE <-wrote<-authors.name ?~ $0"
    );

    // List results
    println!("Books by people named 'George':");
    for s in books_by_george {
        println!("  {}\t{}", s.title, s.word_count.unwrap_or_default())
    }

    // Spacing for terminal ease of reading
    println!("");

    // Run a transaction (errors are automatically bubbled)
    sdb::trans_act!( ( client ) => {
        // Store results of a query in a transaction variable.
        // Queries that follow can act on these results
        $longest = "SELECT * FROM books ORDER word_count DESC";

        // Get just the title of the first result of `$longest`
        longest_title: String = pluck("title", 1) "$longest";

        // Get the number of books in total
        long_count: i32 = count() "$longest";

        // Retrieve all of the books
        stories: Vec<BookSchema> = "SELECT * FROM books";
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
        LevelFilter::Warn,
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
