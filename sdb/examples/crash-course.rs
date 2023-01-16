use sdb::prelude::*;
use serde::{Deserialize, Serialize};


#[tokio::main]
async fn main() -> SdbResult<()> {
    // Create a SurrealClient
    // - Server at 127.0.0.1:8000 connect using websockets 
    // - login as demo_user:demo_pass
    // - using namespace 'example' and database 'demo'
    let client = SurrealClient::open("ws://demo_user:demo_pass@127.0.0.1:8000/example/demo")
        .build()
        .unwrap();


    // Run a query
    let mut results = client.transaction()
        .push("SELECT * FROM 12")
        .run()
        .await?;
    let twelve = results.next_one::<usize>()?;
    assert_eq!(twelve, 12);


    // Use the query! macro to reduce boilerplate
    sdb::query!( client => {
        "SELECT * FROM 12" => twelve_again: usize;
    });
    assert_eq!(twelve, twelve_again);


    // Alternative syntax for single queries
    let twelve_yet_again = sdb::query!( client => { "SELECT * FROM 12" as usize });
    assert_eq!(twelve_again, twelve_yet_again);
    assert_eq!(twelve_yet_again, 12); // just to be sure
    


    // Now run multiple queries
    sdb::query!( client => {
        // Update some records, and return zero results
        "UPDATE books SET word_count = 0 WHERE word_count = unset";

        // Parse results as `Vec<Book>` into rust variable `long_books`
        "SELECT * FROM books WHERE word_count > 250000 FETCH author"
            => long_books: Vec<Book>;
    });
    println!("Here are some long books:");
    for book in long_books {
        println!("  - {:<30} by {:<20} has {} words ", 
            book.title,
            book.author().name,
            book.word_count
        )
    }


    // Inject variables into the query
    let search = "George";
    sdb::query!( client =[ search, 5 ]=> {
        // Use variable
        "SELECT * FROM books WHERE author.name ~ $0" => books_by: Vec<Book>;

        // Use variable by name
        "SELECT * FROM books WHERE author.name !~ $search" => books_not_by: Vec<Book>;

        // Store query in a transaction variable then use it later
        "SELECT * FROM books WHERE author.name ~ $search" => $books_by;
        "SELECT * FROM $books_by LIMIT $1" => _five_books_by: Vec<Book>;
    });
    println!("\n{:?} published {} books", search, books_by.len());
    println!("People not named {} published {} books", search, books_not_by.len());


    // Use Query Sugar™
    let search = "George";
    sdb::query!( client =[ search ]=> {
        // Add up the values of `word_count` in all books
        "SELECT * FROM books WHERE author.name ~ $search"
            .sum("word_count") => total_word_count: usize;

        // Nothing new here
        "SELECT * FROM books WHERE author.name ~ $search" => $books_by;

        // Query Sugar™s can operate on query vars directly
        "$books_by" .count() => author_book_count: usize;
    });
    println!("\n{:?} published {} books with a total word count of {}",
        search,
        author_book_count,
        total_word_count
    );


    // use FETCH clause to get nested data
    sdb::query!( client => {
        // Get 5 random book records and their author field
        "SELECT * FROM books FETCH author"
            .shuffle() .limit( 5 ) => books_by: Vec<Book>;
    });
    println!("\nHere are five books and their authors:" );
    for book in books_by {
        println!("  - {:<30} by {}", book.title, book.author().name )
    }

    // The query! macro will also raise errors at compile time if it 
    // catches a mistake, like:
    // - Clauses out of order
    // - Unbalanced and out of order parenthesies, braces, and brackets
    // - using an undefined variable
    //
    // more checks will be added later

    Ok( () )
}


//
// Schema definition
//

#[derive(Serialize, Deserialize, SurrealRecord)]
#[table("books")]
pub struct Book {
    pub id: RecordId,
    pub title: String,
    pub word_count: usize,
    // either a RecordId, or an Author. This makes FET
    pub author: RecordLink<Author>,
}

#[derive(Serialize, Deserialize, SurrealRecord)]
#[table("authors")]
pub struct Author {
    pub id: RecordId,
    pub name: String,
}

#[derive(Serialize, Deserialize, SurrealRecord)]
#[table("publishers")]
pub struct Publisher {
    pub id: RecordId,
    pub name: String,
}

#[derive(Serialize, Deserialize, SurrealRecord)]
#[table("published")]
pub struct PublishedBy {
    pub id: RecordId,
    pub r#in: RecordLink<Publisher>,
    pub out: RecordLink<Book>,
}