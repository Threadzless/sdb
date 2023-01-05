use std::{sync::PoisonError, any::type_name};
use thiserror::Error;



pub type SdbResult<T> = Result<T, SdbError>;


#[derive(Debug, Error)]
pub enum SdbError {
    // #[error("data store disconnected")]
    // Disconnect(#[from] io::Error),

    #[error("Mutex poisoned, tried to get {0:?}. This means a thread with the mutex locked paniced, probably :(")]
    MutexFailure(String),

    #[error("Invalid host string: {0:?}, expected \"<host>/<namespace>/<database>\" with optiona protocol and port")]
    InvalidHostString( String ),

    #[error("Unable to parse {0:?} as a RecordId")]
    UnableToParseAsRecordId( String ),

    #[error("No SurrealDB server found at {url:?}")]
    ConnectionRefused {
        url: String,
    },

    #[error("The authentication credentials provided are not valid.")]
    AuthenticationError,

    #[error("invalid header {header:?} (expected {expected:?}, found {found:?})")]
    InvalidHeader {
        header: String,
        expected: String,
        found: String,
    },

    #[error("SurrealDB responded with an error message: {0}")]
    Surreal( String ),

    #[error("Missing header {header:?} (expected {expected:?})")]
    MissingHeader {
        header: String,
        expected: String,
    },

    #[error("Ran out of input before expected: {0}")]
    UnexpectedEndOfInput( String ),

    #[error("JSON Parsing error: {0:?}")]
    Serde( #[from] serde_json::Error ),

    // sockets

    #[cfg(all(feature = "ws", target_family = "wasm"))]
    #[error("Connection failed: {0}")]
    SocketError( #[from] gloo_net::websocket::WebSocketError ),

    #[cfg(all(feature = "ws", not(target_family = "wasm")))]
    #[error("Websocket Error: {0:?}")]
    SocketError( websockets::WebSocketError ),
}

impl<Guard> From<PoisonError<Guard>> for SdbError {
    fn from(_value: PoisonError<Guard>) -> Self {
        let type_name = type_name::<Guard>().to_string();
        SdbError::MutexFailure( type_name )
    }
}

#[cfg(all(feature = "ws", not(target_family = "wasm")))]
impl From<websockets::WebSocketError> for SdbError {
    fn from(value: websockets::WebSocketError) -> Self {
        use websockets::WebSocketError as WSErr;
        match value {
            WSErr::HandshakeFailedError { status_code, .. } 
                if status_code.eq("403") => SdbError::AuthenticationError,
            _ => SdbError::SocketError( value ),
        }
    }
}