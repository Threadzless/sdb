use sdb::prelude::*;
use serde::{Deserialize, Serialize};
use simplelog::*;

async fn run() -> Result<(), SdbError> {
    let client = SurrealClient::new("127.0.0.1:8000/example/demo")
        .auth_basic("test_user", "test_pass")
        .protocol(Protocol::Http)
        .build()?;

    sdb::trans_act!( ( client ) => {
        $longest = "SELECT * FROM books ORDER word_count DESC";

        mut longest_titles: Vec<String> =
            pluck("title") "SELECT * FROM $longest LIMIT 3";

        long_count: i32 = count() "$longest";

        stories: Vec<BookSchema> = "SELECT * FROM $longest";
    });



    longest_titles.push("Blah blab".to_string());
    println!("Longest books: {:?}\n", longest_titles);

    println!("All books (ever) {}:", long_count);
    for s in stories {
        println!("  {}", s.title)
    }

    //
    //
    //

    println!("");

    sdb::trans_act!( (client, "George") => {
        good_books: Vec<BookSchema> =
            "SELECT * FROM books WHERE <-wrote<-authors.name ?~ $0"
    });

    println!("Books by people named 'George':");
    for s in good_books {
        println!("  {}", s.title)
    }

    Ok(())
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BookSchema {
    pub id: RecordId,
    pub title: String,
    pub published: Option<String>,
    pub page_count: Option<u32>,
}

//
//
//
//
//

fn main() {
    TermLogger::init(
        LevelFilter::Warn,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();

    let pool = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    pool.block_on(run()).unwrap();
}
