mod request;
mod response;

pub use request::*;
pub use response::*;

use crate::{
    error::SdbResult,
    server_info::ServerInfo
};

#[async_trait::async_trait(?Send)]
pub trait SurrealInterface: Send + Sync {
    async fn send(&mut self, server: &ServerInfo, request: SurrealRequest) -> SdbResult<SurrealResponse>;
}

pub trait SurrealInterfaceBuilder: SurrealInterface {
    fn new(server: &ServerInfo) -> SdbResult<Self>
    where
        Self: Sized;
}
