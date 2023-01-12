use sdb::prelude::*;
use serde::{Deserialize, Serialize};

async fn run() -> Result<(), SdbError> {
    // Open a new client
    let client = SurrealClient::open("127.0.0.1:8000/example/demo")
        .auth_basic("test_user", "test_pass")
        .protocol(Protocol::Socket { secure: false })
        .build()?;


    //
    //
    // Run a simple query
    sdb::query!( (client) => {
        "SELECT * FROM books ORDER BY rand() LIMIT 5" => five_random_books: Vec<BookSchema>
    });
    println!("1) Five books:");
    for book in five_random_books {
        println!("   - {}", book.title)
    }
    println!("");

    //
    //
    // Introduce a variable
    let search_term = "George";
    sdb::query!( (client, search_term) => {
        "SELECT * FROM books WHERE author.name ~ $0 LIMIT 3" => books_by_george: Vec<BookSchema>
    });
    println!("2) Three books by someone named '{}'", search_term);
    for book in books_by_george {
        println!("   - {}", book.title)
    }
    println!("");

    //
    //
    // Fetch nested data (same schema struct can be used)
    let some_books = sdb::query!( (client) => {
        "SELECT * FROM books LIMIT 3 FETCH author" as Vec<BookSchema>
    });
    println!("3) Three books and their authors:");
    for book in some_books {
        println!("   - '{}' by {}", book.title, book.author().name)
    }
    println!("");

    //
    //
    // Use a sugar function
    let short_book_count = sdb::query!( ( client ) => {
        "SELECT * FROM books WHERE word_count < 75_000" .count() as i32
    });
    println!("4) There are {short_book_count} books in the archive with fewer than 75k words");
    println!("");

    //
    //
    // Break up complex queries into multiple lines
    let longest_book_title = sdb::query!( ( client ) => {
        // Store first query result in a transaction variable
        "SELECT * FROM books ORDER word_count DESC" => $longest;

        // Run a query on the results
        "SELECT title FROM $longest LIMIT 1" as String
    });
    println!("5) The longest book is called {longest_book_title:?}");
    println!("");
    
    //
    //
    // Run a multiple queries together.
    // Unlike single queries, 
    let long_word_count = 250000;
    sdb::query!( ( client, long_word_count ) => {
        // Store results of a query in a transaction variable.
        // Queries that follow can act on these results
        "SELECT * FROM books WHERE word_count > $0 ORDER word_count DESC" => $longest;

        // Get just the title of the first result of `$longest`, aka, the book
        // with the most words, and store it in
        "SELECT title FROM $longest LIMIT 1" => longest_title: String;

        // Get the number of books in total with more than 250k words
        "SELECT * FROM $longest" . count() => long_count: i32;

        // Retrieve all of the books, with nested data
        "SELECT * FROM books FETCH author" .limit(10) => stories: Vec<BookSchema>;
    });

    println!("6) Longest books: '{longest_title}'");
    println!("   There are {long_count} books with over {long_word_count} words");
    println!("   Several books:");
    for s in stories {
        println!("    - '{}' by {}", s.title, s.author().name)
    }
    println!("");

    Ok(())
}

//
// Schema Definition
//

#[derive(Serialize, Deserialize, SurrealRecord)]
#[table("books")]
pub struct BookSchema {
    pub id: RecordId,
    pub title: String,
    pub word_count: Option<usize>,
    pub author: RecordLink<AuthorSchema>,
}

#[derive(Serialize, Deserialize, SurrealRecord)]
#[table("authors")]
pub struct AuthorSchema {
    pub id: RecordId,
    pub name: String,
}

//
//
//

#[tokio::main]
async fn main() {
    use simplelog::*;

    // Logging
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();

    run().await.unwrap();
}
