mod method;
mod request;
mod response;

pub use method::*;
pub use request::*;
pub use response::*;

use crate::{error::SdbResult, server_info::ServerInfo};

#[async_trait::async_trait(?Send)]
pub trait SurrealInterface: Send + Sync {
    async fn execute(
        &mut self,
        server: &ServerInfo,
        request: SurrealRequest,
    ) -> SdbResult<SurrealResponse>;

    async fn execute_list(
        &mut self,
        server: &ServerInfo,
        requests: Vec<SurrealRequest>,
    ) -> Vec<SdbResult<SurrealResponse>> {
        let mut responses = Vec::new();
        for req in requests {
            responses.push(
                self.execute(server, req).await
            )
        }
        responses
    }
}

pub trait SurrealInterfaceBuilder: SurrealInterface {
    fn new(server: &ServerInfo) -> SdbResult<Self>
    where
        Self: Sized;
}
