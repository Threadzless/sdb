#![warn(missing_docs)]

/*!
Surreal Data Base client
=========================

An unofficial official client for SurrealDb, with convienience macros and compile time syntax checking.

*/

#[cfg(test)]
#[allow(unused)]
use serde_json::Value;
#[cfg(test)]
#[allow(unused)]
use serde::Serialize;

// pub use sdb_macros::*;

pub use sdb_base::*;

// documentation links
#[allow(unused_imports)]
use sdb_base as sdb;
#[allow(unused_imports)]
use sdb_base::prelude::*;


pub use sdb_macros::query;

pub use sdb_macros::SurrealRecord;


/// This is the only thing you need to import.
/// 
/// 
/// ```rust
/// # use sdb::prelude::*;
/// # use serde::Deserialize;
/// #
/// # async fn test_main() -> SdbResult<()> {
/// let client = SurrealClient::open("ws://test_user:test_pass@127.0.0.1:8000/example/demo")
///     .build()?;
/// 
/// let author_name = "George";
/// let some_books = sdb::query!( (client, author_name) => {
///     "SELECT * FROM books WHERE author.name ~ $0 ORDER BY rand() LIMIT 5 FETCH author" -> Vec<BookSchema>
/// });
/// 
/// println!("Books by people named '{author_name}':");
/// for book in some_books {
///     println!("  - '{}', by {}", book.title, book.author().name);
/// }
/// # Ok(())
/// # }
/// # tokio_test::block_on(async {
/// #     test_main().await.unwrap()
/// # });
/// 
/// #[derive(Deserialize, SurrealRecord)]
/// #[table("books")]
/// struct BookSchema {
///     pub id: RecordId,
///     pub title: String,
///     pub word_count: usize,
///     pub summary: Option<String>, // <- Still parses correctly if field not in query results
///     pub author: RecordLink<AuthorSchema>
/// }
/// #[derive(Deserialize, SurrealRecord)]
/// #[table("books")]
/// struct AuthorSchema {
///     pub id: RecordId,
///     pub name: String,
/// }
/// ```
pub mod prelude {
    pub use sdb_base::prelude::*;

    pub use sdb_macros::*;

    pub use crate::query;
}

#[cfg(doctest)]
pub use sdb_base::example;

#[doc = include_str!("../../README.md")]
#[cfg(doctest)]
pub struct ReadmeMd { }





