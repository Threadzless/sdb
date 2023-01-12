use std::sync::{Arc, Mutex};

use crate::{
    error::SdbResult, reply::TransactionReply, server_info::ServerInfo,
    transaction::TransactionBuilder,
};

mod builder;
pub mod interface;

pub use builder::*;
pub use interface::*;

/// The URL to access the demo database. See [`demo()`](fn@SurrealClient::demo)
const DEMO_URL: &str = "ws://test_user:test_pass@127.0.0.1:8000/example/demo";

#[derive(Clone)]
pub struct SurrealClient {
    inner: Arc<ClientInner>,
}

impl SurrealClient {
    pub fn open(url: &str) -> ClientBuilder {
        ClientBuilder::new(url)
    }

    /// Create a client for accessing the demo database, and polls it to ensure its running.
    /// 
    /// The demo database is launched by running [`launch-demo-db.sh`](launch_demo-db.sh)
    ///
    /// This method is to make tests and demos more convienient, and shouldn't be used in outside of those cases.
    #[cfg(any(test, doctest, feature = "extras"))]
    pub async fn demo() -> Self {
        let Ok( me ) = Self::open(DEMO_URL).build() else {
            panic!("\n\nYou forgot to start the demo server\n ./launch-demo-db.sh")
        };

        me.transaction()
            .push("INFO FOR DB")
            .run()
            .await
            .unwrap();
        me
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
        let full_sql = sqls.join(";\n\t");

        #[cfg(feature = "log")]
        log::info!("Sending Query: \n\t{}\n", full_sql);

        let request = SurrealRequest::query(full_sql);
        let req_id = request.id.clone();

        let mut socket = self.inner.socket.lock().unwrap();

        let response = socket.send(&self.inner.server, request).await?;
        if !response.check_id(&req_id) {
            unreachable!(
                "Packets recieved out of order. {:?} {req_id:?}. Plz report to github",
                response.id()
            )
        }

        match response {
            SurrealResponse::Error { error, .. } => {
                #[cfg(feature = "log")]
                log::error!("SurrealDB response: {:?}", error);

                // eprint!("SurrealDB response: {:#?}", error);

                panic!("Surreal Responded with an error");
            }
            SurrealResponse::Result { result, .. } => match result {
                Some(res) => Ok(TransactionReply::new(queries, res)),
                None => {
                    panic!("~ ~\n{result:?}\n")
                }
            },
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
