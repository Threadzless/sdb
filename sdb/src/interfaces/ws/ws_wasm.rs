use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};
use serde_json::from_str;
use std::fmt::Debug;

// use wasm_sockets::{Message, ConnectionStatus, PollingClient};

use crate::{
    server_info::ServerInfo,
    client::interface::*,
    error::*,
};

impl Debug for WSSurrealInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebSocketProtocol").finish()
    }
}

pub struct WSSurrealInterface {
    socket: Option<WebSocket>,
}

impl WSSurrealInterface {
    pub fn new(_info: &ServerInfo) -> Self {
        Self { socket: None }
    }

    async fn ensure_connected(&mut self, info: &ServerInfo) -> SdbResult<()> {
        if self.socket.is_none() {
            let socket = WebSocket::open(&info.full_url()).unwrap();
            self.socket = Some(socket);
        }

        if let Some(socket) = &mut self.socket {
            let req = SurrealRequest::use_ns_db(&info.namespace, &info.database);
            let msg = req.stringify().unwrap();
            socket.send(Message::Text(msg)).await.unwrap();

            if info.auth.is_some() {
                let req = SurrealRequest::new_auth(&info);
                let msg = req.stringify().unwrap();
                socket.send(Message::Text(msg)).await.unwrap();
                let _reply = socket.next().await.unwrap().unwrap();
            }

            let _reply = socket.next().await.unwrap().unwrap();
        }

        Ok(())
    }
}

impl SurrealInterfaceBuilder for WSSurrealInterface {
    fn new(_server: &ServerInfo) -> SdbResult<Self>
    where
        Self: Sized,
    {
        Ok(Self { socket: None })
    }
}

#[async_trait(?Send)]
impl SurrealInterface for WSSurrealInterface {


    async fn send(&mut self, info: &ServerInfo, request: SurrealRequest) -> SdbResult<SurrealResponse> {
        self.ensure_connected(info).await?;
        let msg = request.stringify();

        #[cfg(feature = "log")]
        log::trace!("Sending => {}", msg);

        let msg = Message::Text( msg );
        let socket = self.socket.as_mut().expect("Socket to be initialized");

        socket.send( msg ).await.unwrap();
        let reply = socket.next().await.unwrap().unwrap();
        let Message::Text( txt ) = reply else { panic!() };

        match from_str::<SurrealResponse>( &txt ) {
            Ok( result ) if ! result.check_id( &request.id ) => {
                unimplemented!("Multi-query routing")
            },
            Ok( r ) => Ok( r ),
            Err( err ) => {
                Err( err.into() )
            }
        }
    }

    // async fn query(&mut self, info: &ServerInfo, sql: String) -> SdbResult<Vec<QueryReply>> {
    //     self.connect_if_not(&info).await.unwrap();

    //     // specify desired ns/db (header support not allowed)
    //     //let sql = format!("USE NS {} DB {};\n{sql}", info.namespace, info.database);

    //     let req = SocketRequest::query( sql );
    //     let msg = req.stringify().unwrap();

    //     #[cfg(feature = "log")]
    //     log::trace!("Sending => {}", msg);

    //     let msg = Message::Text( msg );
    //     let socket = self.socket.as_mut().expect("Socket to be initialized");

    //     socket.send( msg ).await.unwrap();
    //     let reply = socket.next().await.unwrap();
    //     let Message::Text( txt ) = reply? else { panic!() };

    //     match from_str::<SocketResponse>( &txt ) {
    //         Ok( result ) if ! result.check_id( &req.id ) => {
    //             unimplemented!("Multi-query routing")
    //         },
    //         Ok( SocketResponse::Result { result, .. } ) => {
    //             Ok( result )
    //         },
    //         Ok( SocketResponse::Error { .. } ) => {
    //             todo!()
    //         },
    //         Err( err ) => {
    //             Err( err.into() )
    //         }
    //     }
    // }
}
