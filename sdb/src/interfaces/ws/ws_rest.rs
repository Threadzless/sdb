use std::fmt::Debug;

use websockets::{Frame, WebSocket, WebSocketBuilder, WebSocketError};

use crate::{
    client::interface::*,
    error::{SdbError, SdbResult},
    server_info::ServerInfo,
};

pub struct WSSurrealInterface {
    connected_to: Option<ServerInfo>,
    builder: Option<WebSocketBuilder>,
    socket: Option<WebSocket>,
}


impl SurrealInterfaceBuilder for WSSurrealInterface {
    fn new(server: &ServerInfo) -> SdbResult<Self>
    where
        Self: Sized,
    {
        let mut builder = WebSocket::builder();

        for (k, v) in server.headers() {
            builder.add_header(&k, &v);
        }

        Ok(Self {
            connected_to: None,
            builder: Some(builder),
            socket: None,
        })
    }
}

impl WSSurrealInterface {
    async fn ensure_connected(&mut self, server: &ServerInfo) -> SdbResult<()> {
        // Connect to socket if not already connected
        match &mut self.builder {
            Some(builder) => {
                let url = server.full_url();
                let socket = builder.connect(&url).await.unwrap();
                self.builder = None;
                self.socket = Some(socket);
            }
            None => {}
        }

        // Handshake, or rehandshake, with 
        if self.connected_to.is_none() || self.connected_to.as_ref().unwrap().ne( server ) {
            let socket = self.socket.as_mut().unwrap();// else { panic!() };
            let req = SurrealRequest::new_auth(server);
            let txt = serde_json::to_string(&req).unwrap();
            socket.send( Frame::text( txt ) ).await.unwrap();
            recieve_next(socket).await.unwrap();
            self.connected_to = Some( server.clone() );

            Ok( () )
        }
        else {
            Ok( () )
        }
    }
}

#[async_trait::async_trait(?Send)]
impl SurrealInterface for WSSurrealInterface {
    async fn send(&mut self, server: &ServerInfo, request: SurrealRequest) -> SdbResult<SurrealResponse> {
        self.ensure_connected(server).await?;
        
        let socket = self.socket.as_mut().unwrap();

        socket
            .send_text(serde_json::to_string(&request).unwrap())
            .await
            .unwrap();

        let frame = recieve_next(socket).await?;
        
        let Frame::Text { payload, .. } = frame else {
            panic!("Recieved a Binary payload - WSSurrealInterface::send")
        };

        #[cfg(feature = "log")]
        log::info!("SurrealDB response: \n{}", payload);

        let response = serde_json::from_str::<SurrealResponse>(&payload).unwrap();
        match &response {
            SurrealResponse::Error { error, .. } => {
                return Err(SdbError::Surreal(error.message.clone()))
            }
            SurrealResponse::Result { id, ..} => {
                if id.ne(&request.id) {
                    panic!()
                }
            }
        }

        Ok(response)
    }
}

impl Debug for WSSurrealInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebSocketProtocol").finish()
    }
}

async fn recieve_next(sock: &mut WebSocket) -> Result<Frame, WebSocketError> {
    loop {
        let reply = sock.receive().await?;
        if let Frame::Text { .. } = reply {
            return Ok(reply);
        } else if let Frame::Binary { .. } = reply {
            return Ok(reply);
        }
    }
}
