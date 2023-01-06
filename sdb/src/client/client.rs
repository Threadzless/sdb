use std::sync::{Arc, Mutex};

use crate::{
    // protocols::SdbProtocol,
    client::{ClientBuilder, SurrealInterface},
    error::SdbResult,
    reply::TransactionReply,
    server_info::ServerInfo,
    transaction::TransactionBuilder,
};

use super::{SurrealInterfaceBuilder, SurrealRequest, SurrealResponse};

#[derive(Clone)]
pub struct SurrealClient {
    inner: Arc<ClientInner>,
}

impl SurrealClient {
    pub fn new(url: &str) -> ClientBuilder {
        ClientBuilder::new(url)
    }

    pub fn server(&self) -> &ServerInfo {
        &self.inner.server
    }

    pub(crate) fn build<I: SurrealInterfaceBuilder + 'static>(
        server: ServerInfo,
    ) -> SdbResult<Self> {
        let socket = Box::new(Mutex::new(I::new(&server)?));

        let inner = Arc::new(ClientInner {
            socket,
            server,
        });

        Ok(Self { inner })
    }

    pub fn transaction(&self) -> TransactionBuilder {
        TransactionBuilder::new(self)
    }

    pub async fn query(&mut self, trans: TransactionBuilder) -> SdbResult<TransactionReply> {
        let (queries, sqls) = trans.queries();
        let full_sql = sqls.join(";\n");

        let request = SurrealRequest::query(full_sql);
        let req_id = request.id.clone();

        let mut socket = self.inner.socket.lock().unwrap();

        let response = socket.send(&self.inner.server, request).await;
        match response.unwrap() {
            SurrealResponse::Error { id, error } => {
                if id.ne(&req_id) {
                    unreachable!(
                        "Packets recieved out of order. {id:?} {req_id:?}. Plz report to github"
                    )
                }

                #[cfg(feature = "log")]
                log::error!("SurrealDB response: {:?}", error);

                eprint!("SurrealDB response: {:?}", error);

                panic!("Surreal Responded with an error");
            }
            SurrealResponse::Result { id, result } => {
                if id.ne(&req_id) {
                    unreachable!(
                        "Packets recieved out of order. {id:?} {req_id:?}. Plz report to github"
                    )
                }

                let results = result.expect("A non-null transaction response");

                Ok(TransactionReply::new(queries, results))
            }
        }
    }
}

//
//
//

unsafe impl Sync for ClientInner { }
unsafe impl Send for ClientInner { }
pub struct ClientInner {
    socket: Box<Mutex<dyn SurrealInterface>>,
    server: ServerInfo,
}
