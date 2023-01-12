#![feature(let_chains, if_let_guard)]

mod client;
mod interfaces;
mod reply;
mod transaction;

mod credentials;
mod error;
mod protocol;
mod record;
mod server_info;

pub mod prelude {

    pub use crate::{
        client::interface::{SurrealRequest, SurrealResponse, SurrealResponseError},
        client::SurrealClient,
        credentials::Credentials,
        error::{SdbError, SdbResult},
        protocol::Protocol,
        record::*,
        reply::TransactionReply,
        server_info::ServerInfo,
        transaction::TransactionBuilder,
    };
}

/// A little helper for de-cluttering the examples code in [README.md](README.md)
/// and other places
#[cfg(any(feature = "extras", test, doctest))]
#[macro_export]
macro_rules! example {
    { $($arg:tt)+ } => {
        use sdb::prelude::*;
        use serde::Deserialize;

        tokio_test::block_on( async {
            main_test( ).await.unwrap()
        });

        async fn main_test( ) -> SdbResult<()> {
            let client = SurrealClient::demo();
            $($arg)+
            ;
            Ok( () )
        }

        #[derive(Deserialize, SurrealRecord)]
        #[table("books")]
        struct BookSchema {
            pub id: RecordId,
            pub title: String,
            pub word_count: usize,
            pub summary: Option<String>,
            pub author: RecordLink<AuthorSchema>,
        }

        #[derive(Deserialize, SurrealRecord)]
        #[table("authors")]
        struct AuthorSchema {
            pub id: RecordId,
            pub name: String,
            pub is_alive: Option<bool>,
        }
    };
}


