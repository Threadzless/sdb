#![feature(let_chains, if_let_guard)]

pub use ::sdb_macros::*;
pub use ::serde_json::Value;

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
    pub use sdb_macros::SurrealRecord;
    pub use crate::{
        client::interface::{SurrealRequest, SurrealResponse, SurrealResponseError, RequestMethod},
        client::SurrealClient,
        credentials::Credentials,
        error::{SdbError, SdbResult},
        protocol::Protocol,
        record::*,
        reply::QueryReply,
        server_info::ServerInfo,
        transaction::TransactionBuilder,
    };
}


#[doc = include_str!("../../README.md")]
#[allow(unused)]
struct ReadMe { }


/// A little helper for de-cluttering the examples code in [README.md](README.md)
/// and other places
#[macro_export]
macro_rules! doctest {
    { $client: ident => { $($arg:tt)+ }} => {
        sdb::doctest!{
            let $client = SurrealClient::demo();
            $($arg)+
        }
    };
    { $($arg:tt)+ } => {
        use sdb::prelude::*;
        use serde::*;

        tokio_test::block_on( async {
            main_test( ).await.unwrap()
        });

        async fn main_test() -> SdbResult<()> {
            $($arg)+
            ;
            Ok( () )
        }

        #[derive(Serialize, Deserialize, SurrealRecord)]
        #[table("books")]
        struct Book {
            pub id: RecordId,
            pub title: String,
            pub word_count: Option<usize>,
            pub summary: Option<String>,
            pub author: RecordLink<Author>,
        }

        #[derive(Serialize, Deserialize, SurrealRecord)]
        #[table("authors")]
        struct Author {
            pub id: RecordId,
            pub name: String,
            pub is_alive: Option<bool>,
        }
    };
}


