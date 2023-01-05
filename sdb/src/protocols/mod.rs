use async_trait::async_trait;
use std::fmt::Debug;

use crate::{
    reply::QueryReply,
    server_info::ServerInfo,
    error::SdbResult,
};

#[cfg(feature = "http")]
mod http;
#[cfg(feature = "http")]
pub use http::*;

#[cfg(feature = "ws")]
mod ws;
#[cfg(feature = "ws")]
pub use ws::*;


#[async_trait(?Send)]
pub trait SdbProtocol: Debug {
    async fn connect_if_not(&mut self, info: &ServerInfo) -> SdbResult<()>;
    async fn query(&mut self, info: &ServerInfo, sql: String) -> SdbResult<Vec<QueryReply>>;
}

// #[async_trait(?Send)]
// pub trait SdbProtocol<R: Read>: Debug {
//     async fn connect(&mut self) -> Result<BufReader<R>, SdbError>;
//     async fn query(&mut self, sql: String) -> Result<BufReader<R>, SdbError>;
// }



// #[async_trait(?Send)]
// pub trait SdbProtocol: Debug {
//     async fn connect(&mut self) -> Result<&str, SdbError>;
//     async fn query(&mut self, sql: String) -> Result<&str, SdbError>;
// }

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
