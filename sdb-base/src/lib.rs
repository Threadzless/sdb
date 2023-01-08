#![feature(let_chains, if_let_guard)]
#![allow(unused)]

pub mod client;
pub mod interfaces;
// pub mod protocols;
pub mod reply;
pub mod transaction;

pub mod any_record;
pub mod credentials;
pub mod error;
pub mod record;
pub mod record_id;
pub mod record_link;
pub mod server_info;

pub mod prelude {

    pub use crate::{
        any_record::AnyRecord,
        client::SurrealClient,
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
        Tikv,
    }
}
