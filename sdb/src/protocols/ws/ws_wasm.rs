use std::fmt::Debug;
use async_trait::async_trait;
use serde_json::from_str;
use futures::{SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};

// use wasm_sockets::{Message, ConnectionStatus, PollingClient};

use super::*;
use crate::{
    reply::QueryReply,
    error::SdbResult,
    server_info::ServerInfo
};


impl Debug for WebSocketProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebSocketProtocol").finish()
    }
}

pub struct WebSocketProtocol {
    socket: Option<WebSocket>,
}

unsafe impl Sync for WebSocketProtocol { }
unsafe impl Send for WebSocketProtocol { }

impl WebSocketProtocol {
    pub fn new( _info: &ServerInfo) -> Self {
        Self {
            socket: None,
        }
    }
}

#[async_trait(?Send)]
impl super::SdbProtocol for WebSocketProtocol {
    async fn connect_if_not(&mut self, info: &ServerInfo) -> SdbResult<()> {
        
        if self.socket.is_none() {
            let socket = WebSocket::open(&info.full_url()).unwrap();
            self.socket = Some( socket );
        }

        if let Some( socket ) = &mut self.socket {
            let req = SocketRequest::use_ns_db( &info.namespace, &info.database);
            let msg = req.stringify().unwrap();
            socket.send(Message::Text(msg)).await?;

            if info.auth.is_some() {
                let req = SocketRequest::new_auth( &info );
                let msg = req.stringify().unwrap();
                socket.send(Message::Text(msg)).await?;
                let _reply = socket.next().await.unwrap()?;
            }

            let _reply = socket.next().await.unwrap()?;
        }

        Ok( () )
    }

    async fn query(&mut self, info: &ServerInfo, sql: String) -> SdbResult<Vec<QueryReply>> {
        self.connect_if_not(&info).await.unwrap();

        // specify desired ns/db (header support not allowed)
        //let sql = format!("USE NS {} DB {};\n{sql}", info.namespace, info.database);

        let req = SocketRequest::query( sql );
        let msg = req.stringify().unwrap();

        #[cfg(feature = "log")]
        log::trace!("Sending => {}", msg);

        let msg = Message::Text( msg );
        let socket = self.socket.as_mut().expect("Socket to be initialized");

        socket.send( msg ).await.unwrap();
        let reply = socket.next().await.unwrap();
        let Message::Text( txt ) = reply? else { panic!() };

        match from_str::<SocketResponse>( &txt ) {
            Ok( result ) if ! result.check_id( &req.id ) => {
                unimplemented!("Multi-query routing")
            },
            Ok( SocketResponse::Result { result, .. } ) => {
                Ok( result )
            },
            Ok( SocketResponse::Error { .. } ) => {
                todo!()
            },
            Err( err ) => {
                Err( err.into() )
            }
        }
    }   
}