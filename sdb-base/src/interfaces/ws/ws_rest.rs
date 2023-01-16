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
                let socket = convert_err(builder.connect(&url).await, server)?;
                self.builder = None;
                self.socket = Some(socket);
            }
            None => {}
        }

        // Handshake, or rehandshake, with
        if self.connected_to.is_none() || self.connected_to.as_ref().unwrap().ne(server) {
            let socket = self.socket.as_mut().unwrap(); // else { panic!() };
            let req = SurrealRequest::new_auth(server);
            let txt = serde_json::to_string(&req).unwrap();
            socket.send(Frame::text(txt)).await.unwrap();
            recieve_next(socket).await;
            self.connected_to = Some(server.clone());

            Ok(())
        } else {
            Ok(())
        }
    }
}

unsafe impl Send for WSSurrealInterface {}
unsafe impl Sync for WSSurrealInterface {}

#[async_trait::async_trait(?Send)]
impl SurrealInterface for WSSurrealInterface {
    async fn execute(
        &mut self,
        server: &ServerInfo,
        request: SurrealRequest,
    ) -> SdbResult<SurrealResponse> {
        self.ensure_connected(server).await?;

        let socket = self.socket.as_mut().unwrap();

        socket.send_text(request.stringify()).await.unwrap();

        let frame = recieve_next(socket).await;

        let Frame::Text { payload, .. } = frame else {
            panic!("Recieved a Binary payload - WSSurrealInterface::send")
        };

        let response = match serde_json::from_str::<SurrealResponse>(&payload) {
            Ok(r) => r,
            Err(err) => {
                println!("{payload}\n");
                Err(err).unwrap()
            }
        };

        if !response.is_for(&request) {
            println!(
                "\n\n{payload}\n\n{:?} =/= {:?}\n\n",
                request.id,
                response.id()
            );
            unreachable!("Responses recieved out of order :( ");
        }

        Ok(response)
    }
}

impl Debug for WSSurrealInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebSocketProtocol").finish()
    }
}

async fn recieve_next(sock: &mut WebSocket) -> Frame {
    loop {
        let reply = sock.receive().await.unwrap();
        if let Frame::Text { .. } = reply {
            return reply;
        } else if let Frame::Binary { .. } = reply {
            return reply;
        }
    }
}

/// Convert [`WebSocketError`] to [`SdbError`],
pub fn convert_err<T>(base: Result<T, WebSocketError>, info: &ServerInfo) -> Result<T, SdbError> {
    use std::io::ErrorKind as IoErr;
    use WebSocketError as WsErr;

    let base_err = match base {
        Ok(val) => return Ok(val),
        Err(e) => e,
    };

    let url = info.full_url();

    Err(match &base_err {
        WsErr::TcpConnectionError(err) => match err.kind() {
            IoErr::ConnectionRefused => SdbError::ConnectionRefused { url },
            IoErr::ConnectionAborted | IoErr::ConnectionReset | IoErr::BrokenPipe => {
                SdbError::ConnectionClosed {
                    url,
                    info: err.kind().to_string(),
                }
            }
            _ => SdbError::WebsocketNetworkError(base_err),
        },
        WsErr::WebSocketClosedError => SdbError::ConnectionClosed {
            url,
            info: "Socket closed".to_string(),
        },
        WsErr::PayloadTooLargeError => SdbError::OversizedPayload,
        _ => SdbError::WebsocketNetworkError(base_err),
    })
}
