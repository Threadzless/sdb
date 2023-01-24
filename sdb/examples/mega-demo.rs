use sdb::prelude::*;
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() -> Result<(), SdbError> {
    // Open a new client
    let client = SurrealClient::open("ws://127.0.0.1:8000/example/demo")
        .auth_basic("demo_user", "demo_pass")
        .build()?;


    // Execute a basic query
    sdb::query!( client => {
        "SELECT * FROM 12" => twelve: isize
    });
    if twelve == 12 {
        println!("1) Twelve does in fact equal 12");
    }

    // Alternative syntax for single values
    let twelve_two = sdb::query!( client => { "SELECT * FROM 12" as isize });
    if twelve_two == 12 {
        println!("   Twelve continues to equal 12");
    }


    // Now multiple queries. Results are parsed with `serde::Deserialize`
    sdb::query!( client => {
        // Run this query, ignore the results.
        "UPDATE books SET word_count = 0 WHERE word_count < 0";

        // Zero or more results
        "SELECT * FROM books ORDER rand()" => all_books: Vec<BookSchema>;

        // Just one. Getting zero results throws an error
        "SELECT * FROM books ORDER rand()" => a_book: BookSchema;

        // Returns the first results, or None if there are 0 results.
        "SELECT * FROM books ORDER rand()" => another_book: Option<BookSchema>;
    });

    println!("2) There are {} books in total", all_books.len());
    println!("   There is, allegedly, a book called {}", a_book.title);
    if let Some( ab ) = another_book {
        println!("   And maybe a book called {}", ab.title);
    }


    // Now store results in a mutable field
    sdb::query!( client => {
        "SELECT * FROM books ORDER rand() LIMIT 5" => mut five_books: Vec<BookSchema> 
    });

    println!("3) Five books:");
    while let Some( book ) = five_books.pop() {
        println!("     - {}", book.title)
    }


    // Fetch nested data
    sdb::query!( client => {
        "SELECT * FROM books LIMIT 3 FETCH author" => some_books: Vec<BookSchema>;
    });
    println!("4) Three books and their authors:");
    for book in some_books {
        println!("     - '{}' by {}", book.title, book.author().name)
    }


    // Inject variables
    let search_term = "George";
    sdb::query!( client =[ search_term, 6 ]=> {
        // $0 = `search_term`,  $1 = 6, and so on
        "SELECT * FROM books WHERE author.name ~ $0 LIMIT $1"
            => by_george: Vec<BookSchema>;

        // passed vars in =[ .. ]=> can also be refered to by name
        "SELECT * FROM books WHERE author.name ~ $search_term LIMIT $1"
            => not_by_george: Vec<BookSchema>;
    });
    println!("5) Books by '{}'", search_term);
    for book in by_george {
        println!("     - {}", book.title)
    }    
    println!("   Books they didn't write:");
    for book in not_by_george {
        println!("     - {}", book.title)
    }

    // Use Query Sugar™
    // See README or macro docs for a list of sugars
    sdb::query!( client => {
        "SELECT * FROM books" .count() => book_count: i32;

        // This does the same as the above,
        "SELECT * FROM count(( SELECT * FROM books ))";
    });
    println!("6) There are {book_count} books in the archive");


    // Break up queries into multiple lines.
    sdb::query!( client => {
        // Store first query result in a transaction variable
        "SELECT * FROM books ORDER BY word_count DESC" => $longest;

        // Run a query on the results
        "SELECT * FROM $longest LIMIT 3" => mut longest_books: Vec<BookSchema>
    });
    let longest = longest_books.remove(0);
    println!("7) The longest book is called '{}'", longest.title);


    // Run multiple queries together.
    let long_word_count = 250000;
    sdb::query!( client =[ long_word_count ]=> {
        // Store results of a query in a transaction variable.
        // Queries that follow can act on these results
        "SELECT * FROM books WHERE word_count > $0 ORDER word_count DESC" => $longest;

        // Get just the title of the first result of `$longest`, aka, the book
        // with the most words, and store it in
        "$longest" .pluck("title", 1) => longest_title: String;

        // Get the number of books in total with more than 250k words
        "SELECT * FROM $longest" .count() => long_count: i32;

        // Retrieve 10 books, with nested data
        "SELECT * FROM books FETCH author" .limit(10) => stories: Vec<BookSchema>;
    });

    println!("8) Longest books: '{longest_title}'");
    println!("   There are {long_count} books with over {long_word_count} words");
    println!("   Several books:");
    for s in stories {
        println!("    - '{}' by {}", s.title, s.author().name)
    }

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