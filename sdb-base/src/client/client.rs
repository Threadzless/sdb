use std::sync::{Arc, Mutex};

use crate::{
    client::{interface::*, ClientBuilder, SurrealInterface},
    error::SdbResult,
    reply::TransactionReply,
    server_info::ServerInfo,
    transaction::TransactionBuilder,
};

/// The URL to access the demo database, which is launched by running `./launch-demo-db.sh`
///
/// See [`SurrealClient`]`::demo()` for more info
pub const DEMO_URL: &str = "ws://test_user:test_pass@127.0.0.1:8000/example/demo";

#[derive(Clone)]
pub struct SurrealClient {
    inner: Arc<ClientInner>,
}

impl SurrealClient {
    pub fn new(url: &str) -> ClientBuilder {
        ClientBuilder::new(url)
    }

    /// Create a client for accessin the demo database. It's only useful for code
    /// examples, and probably shouldn't be used outside of that
    ///
    /// The demo database is launched by running `./launch-demo-db.sh`
    ///
    /// This:
    /// ```rust
    /// # use sdb_base::prelude::*;
    /// #
    /// let client = SurrealClient::demo().unwrap();
    /// ```
    /// Is equivilent to this:
    /// ```rust
    /// # use sdb_base::prelude::*;
    /// #
    /// let client = SurrealClient::new("127.0.0.1:8000/example/demo")
    ///     .auth_basic("test_user", "test_pass")
    ///     .protocol( Protocol::Socket { secure: false } )
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn demo() -> SdbResult<Self> {
        Self::new(DEMO_URL).build()
    }

    pub fn server(&self) -> &ServerInfo {
        &self.inner.server
    }

    pub(crate) fn build<I: SurrealInterfaceBuilder + 'static>(
        server: ServerInfo,
    ) -> SdbResult<Self> {
        let socket = Box::new(Mutex::new(I::new(&server)?));

        let inner = Arc::new(ClientInner { socket, server });

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

        let response = socket.send(&self.inner.server, request).await?;
        if ! response.check_id(&req_id) {
            unreachable!(
                "Packets recieved out of order. {:?} {req_id:?}. Plz report to github", 
                response.id()
            )
        }

        match response {
            SurrealResponse::Error { error, .. } => {
                #[cfg(feature = "log")]
                log::error!("SurrealDB response: {:?}", error);

                eprint!("SurrealDB response: {:?}", error);

                panic!("Surreal Responded with an error");
            }
            SurrealResponse::Result { result, .. } => {
                match result {
                    Some(res) => Ok(TransactionReply::new(queries, res)),
                    None => {
                        panic!("~ ~\n{result:?}\n")
                    }
                }
            }
        }
    }
}

//
//
//

unsafe impl Sync for ClientInner {}
unsafe impl Send for ClientInner {}
pub struct ClientInner {
    socket: Box<Mutex<dyn SurrealInterface>>,
    server: ServerInfo,
}

// enum SdbInterface {
//     Socket( Box<WSSurrealInterface> ),
//     Http( Box<HttpSurrealInterface> )
// }

// impl SdbInterface {
//     pub async fn send( &mut self, server: &ServerInfo, request: SurrealRequest ) -> SdbResult<SurrealResponse> {
//         match self {
//             SdbInterface::Socket( i ) => {
//                 i.as_mut().send(server, request).await
//             },
//             SdbInterface::Http( i ) => {
//                 i.as_mut().send(server, request).await
//             },
//         }
//     }
// }
