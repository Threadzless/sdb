use std::fmt::Debug;
use async_trait::async_trait;

use websockets::{
    WebSocket, WebSocketBuilder,
    Frame, WebSocketError
    
};

use super::*;
use crate::{
    server_info::ServerInfo,
    reply::QueryReply,
    error::SdbError
};

// #[derive(Debug)]
pub struct WebSocketProtocol {
    builder: Option<WebSocketBuilder>,
    socket: Option<WebSocket>,
}

unsafe impl Sync for WebSocketProtocol { }
unsafe impl Send for WebSocketProtocol { }

impl WebSocketProtocol {
    pub fn new( info: &ServerInfo) -> Self {
        let mut builder = WebSocket::builder();

        for (k, v) in info.headers() {
            builder.add_header(&k, &v);
        }

        Self { 
            builder: Some(builder),
            socket: None,
        }
    }
}

#[async_trait(?Send)]
impl SdbProtocol for WebSocketProtocol {
    async fn connect_if_not(&mut self, info: &ServerInfo) -> Result<(), SdbError> {

        match &mut self.builder {
            Some( builder ) =>{
                let url = info.full_url();

                let socket = builder.connect(&url).await?;

                self.builder = None;
                self.socket = Some( socket );
            },
            None => { }
        }
        Ok( () )
    }
    async fn query(&mut self, info: &ServerInfo, sql: String) -> Result<Vec<QueryReply>, SdbError> {
        self.connect_if_not(info).await.unwrap();
        
        let request = SocketRequest::query( sql );
 
        let socket = self.socket.as_mut()
            .ok_or( SdbError::SocketError( WebSocketError::WebSocketClosedError ) )?;

        socket.send_text( serde_json::to_string(&request)? )
            .await?;

        match recieve_next(socket).await? {
            Frame::Text { payload, .. } => {
                let reply: SocketResponse = serde_json::from_str( &payload )?;
                match reply {
                    SocketResponse::Error { error, .. } => {
                        return Err( SdbError::Surreal( error.message ) )
                    },
                    SocketResponse::Result { id, result } => {
                        if id.ne( &request.id ) {
                            panic!()
                        }

                        Ok( result )
                    },
                }
            },
            Frame::Binary { .. } => todo!(),
            _ => unreachable!()
        }
    }   
}


impl Debug for WebSocketProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebSocketProtocol")
            .finish()
    }
}

async fn recieve_next( sock: &mut WebSocket ) -> Result<Frame, WebSocketError> {
    loop {
        let reply = sock.receive().await?;
        if let Frame::Text { .. } = reply {
            return Ok( reply )
        }
        else if let Frame::Binary { .. } = reply {
            return Ok( reply )
        }
    }
}