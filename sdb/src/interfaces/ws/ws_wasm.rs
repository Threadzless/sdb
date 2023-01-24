use futures::{SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};
use serde_json::from_str;
use std::fmt::Debug;

// use wasm_sockets::{Message, ConnectionStatus, PollingClient};

use crate::{client::interface::*, error::*, server_info::ServerInfo};

impl Debug for WSSurrealInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebSocketProtocol").finish()
    }
}

pub struct WSSurrealInterface {
    socket: Option<WebSocket>,
}

impl WSSurrealInterface {
    async fn ensure_connected(&mut self, info: &ServerInfo) -> SdbResult<()> {
        if self.socket.is_none() {
            let socket = WebSocket::open(&info.full_url()).unwrap();
            self.socket = Some(socket);
        }

        if let Some(socket) = &mut self.socket {
            let req = SurrealRequest::use_ns_db(&info.namespace, &info.database);
            let msg = req.stringify();
            socket.send(Message::Text(msg)).await.unwrap();
            if info.auth.is_some() {
                let req = SurrealRequest::new_auth(&info);
                let msg = req.stringify();
                socket.send(Message::Text(msg)).await.unwrap();
                let _ = socket.next().await.unwrap().unwrap();
            }

            let _ = socket.next().await.unwrap();
            // let _reply = socket.next().await.unwrap().unwrap();
        }

        Ok(())
    }
}

impl SurrealInterfaceBuilder for WSSurrealInterface {
    fn new(_info: &ServerInfo) -> SdbResult<Self> {
        Ok(Self { socket: None })
    }
}

unsafe impl Send for WSSurrealInterface {}
unsafe impl Sync for WSSurrealInterface {}

#[async_trait::async_trait(?Send)]
impl SurrealInterface for WSSurrealInterface {
    async fn execute(
        &mut self,
        info: &ServerInfo,
        request: SurrealRequest,
    ) -> SdbResult<SurrealResponse> {
        self.ensure_connected(info).await?;
        let msg = request.stringify();

        let msg = Message::Text(msg);
        let socket = self.socket.as_mut().unwrap(); // else { panic!("Socket to be initialized") };

        let _ = socket.send(msg).await;
        let reply = socket.next().await.unwrap().unwrap();

        let Message::Text( txt ) = reply else { panic!() };

        match from_str::<SurrealResponse>(&txt) {
            Ok(result) if !result.is_for(&request) => {
                unimplemented!("Multi-query routing")
            }
            Ok(r) => Ok(r),
            Err(err) => Err(SdbError::QueryResultParseFailure {
                query: request.params[0].to_string(),
                target_type: "SurrealResponse".to_string(),
                serde_err: err,
                value: None,
            }),
        }
    }

    
}
