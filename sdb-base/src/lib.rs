#![feature(let_chains, if_let_guard)]

mod client;
mod interfaces;
mod reply;
mod transaction;

mod any_record;
mod credentials;
mod error;
mod parse_target;
mod protocol;
mod record;
mod record_id;
mod record_link;
mod server_info;
mod value;

pub mod prelude {

    pub use crate::{
        any_record::AnyRecord,
        client::interface::{SurrealRequest, SurrealResponse, SurrealResponseError},
        client::{interface, SurrealClient},
        credentials::Credentials,
        error::{SdbError, SdbResult},
        parse_target::*,
        protocol::Protocol,
        record::SurrealRecord,
        record_id::RecordId,
        record_link::RecordLink,
        reply::TransactionReply,
        server_info::ServerInfo,
        transaction::TransactionBuilder,
        value::*,
    };
}
