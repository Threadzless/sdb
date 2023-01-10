#![feature(let_chains, if_let_guard, future_join)]

mod client;
mod interfaces;
mod reply;
mod transaction;

mod any_record;
mod credentials;
mod error;
mod record;
mod record_id;
mod record_link;
mod server_info;

pub mod prelude {

    pub use crate::{
        client::SurrealClient,
        client::interface::{SurrealRequest, SurrealResponse, SurrealResponseError},
        reply::TransactionReply,
        transaction::TransactionBuilder,

        any_record::AnyRecord,
        credentials::Credentials,
        error::{SdbError, SdbResult},
        protocols::Protocol,
        record::SurrealRecord,
        record_id::RecordId,
        record_link::RecordLink,
        server_info::ServerInfo,
    };
}

pub mod protocols {
    #[derive(Clone, Debug, PartialEq, Default)]
    pub enum Protocol {
        /// Http POST requests. Slow, but ez.
        Http,

        /// Websockets, faster.
        #[default]
        Socket,

        /// TiKV - scalable distributed storage layer that's surrealDb compatible
        /// 
        /// Not implemented 
        Tikv,
    }
}
