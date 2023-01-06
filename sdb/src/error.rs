use std::{any::type_name, sync::PoisonError};
use std::fmt::{Formatter, Debug, Result as FmtResult};
use reqwest::StatusCode;
use thiserror::Error as ThisError;
use serde_json::Error;



pub type SdbResult<T> = Result<T, SdbError>;


// TODO: rebuild error system from the ground up


// TODO: better errror
#[derive(ThisError)]
pub enum SdbError {

    /// Failed to convert the response from SurrealDB into 
    /// the target type.
    #[error("Failed parse a {parse_target} from {query}\n\n{serde_err}\n")]
    QueryResultParseError {
        parse_target: String,
        serde_err: Error,
        query: String,
    },

    // #[error("data store disconnected")]
    // Disconnect(#[from] io::Error),
    #[error("Mutex poisoned, tried to get {0:?}. This means a thread with the mutex locked paniced, probably :(")]
    MutexFailure(String),

    #[error("Invalid host string: {found:?}, expected \"<host>/<namespace>/<database>\" with optiona protocol and port")]
    InvalidHostString {
        found: String
    },

    #[error("Unable to parse {0:?} as a RecordId")]
    UnableToParseAsRecordId(String),

    #[error("No SurrealDB server found at {url:?}")]
    ConnectionRefused { url: String },

    #[error("The authentication credentials provided are not valid.")]
    AuthenticationError,

    #[error("invalid header {header:?} (expected {expected:?}, found {found:?})")]
    InvalidHeader {
        header: String,
        expected: String,
        found: String,
    },

    #[error("Query Timeout")]
    QueryTimeout,

    #[error("Network Timeout")]
    NetworkTimeout,

    #[cfg(all(feature = "http", not(target_family = "wasm")))]
    #[error("Network Status code: {0:?}")]
    NetworkStatus( Option<StatusCode> ),

    #[cfg(all(feature = "http", target_family = "wasm"))]
    #[error("Network Status code: {0:?}")]
    NetworkStatus( i32 ),

    #[error("SurrealDB responded with an error message: {0}")]
    Surreal(String),

    #[error("Missing header {header:?} (expected {expected:?})")]
    MissingHeader { header: String, expected: String },

    #[error("Ran out of input before expected: {0}")]
    UnexpectedEndOfInput(String),

    #[error("JSON Parsing error: {0:?}")]
    Serde(#[from] serde_json::Error),

    #[cfg(all(feature = "http", not(target_family = "wasm")))]
    #[error("Reqwest error {0:?}")]
    Reqwest( reqwest::Error ),

    // sockets
    #[cfg(all(feature = "ws", target_family = "wasm"))]
    #[error("Connection failed: {0}")]
    SocketError(#[from] gloo_net::websocket::WebSocketError),

    #[cfg(all(feature = "ws", not(target_family = "wasm")))]
    #[error("Websocket Error: {0:?}")]
    SocketError(websockets::WebSocketError),
}

impl<Guard> From<PoisonError<Guard>> for SdbError {
    fn from(_value: PoisonError<Guard>) -> Self {
        let type_name = type_name::<Guard>().to_string();
        SdbError::MutexFailure(type_name)
    }
}

#[cfg(all(feature = "ws", not(target_family = "wasm")))]
impl From<websockets::WebSocketError> for SdbError {
    fn from(value: websockets::WebSocketError) -> Self {
        use websockets::WebSocketError as WSErr;
        match value {
            WSErr::HandshakeFailedError { status_code, .. } if status_code.eq("403") => {
                SdbError::AuthenticationError
            }
            _ => SdbError::SocketError(value),
        }
    }
}

#[cfg(all(feature = "http", not(target_family = "wasm")))]
impl From<reqwest::Error> for SdbError {
    fn from(value: reqwest::Error) -> Self {
        if value.is_timeout() {
            SdbError::NetworkTimeout
        }
        else if value.is_status() {
            SdbError::NetworkStatus( value.status() )
        }
        else {
            SdbError::Reqwest( value )
        }
    }
}

//

//

impl Debug for SdbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::QueryResultParseError { parse_target, serde_err, query } => write!(f, r#"
Unable to Parse query result

The results of this query:
    {query:?}
Weren't able to be parsed as  
   {parse_target}

{serde_err:?}

            "#),

            Self::InvalidHostString { found } => write!(f, r#"
Invalid Connection String
Expected:
    "<host>/<namespace>/<database>"
Found:
    {found:?}
    
    
            "#),

            Self::InvalidHeader { header, expected, found } => write!(f,r#"
Invalid Header on SurrealDB response:
    {header} = {found:?}
Expected:
    {header} = {expected:?}

            "#),

            Self::QueryTimeout => write!(f, "Query Timeout"),

            Self::NetworkTimeout => write!(f, "Network Timeout"),

            Self::NetworkStatus( code ) => write!(f, "Network Status: {code:?}\n\n"),

            Self::ConnectionRefused { url } => write!(f, "Connection refused: {url:?}"),

            Self::AuthenticationError => write!(f, "AuthenticationError"),

            Self::Reqwest(err) => write!(f, "{err:?}"),

            Self::MutexFailure(arg0) => f.debug_tuple("MutexFailure").field(arg0).finish(),
            Self::UnableToParseAsRecordId(arg0) => f.debug_tuple("UnableToParseAsRecordId").field(arg0).finish(),
            Self::Surreal(arg0) => f.debug_tuple("Surreal").field(arg0).finish(),
            Self::MissingHeader { header, expected } => f.debug_struct("MissingHeader").field("header", header).field("expected", expected).finish(),
            Self::UnexpectedEndOfInput(arg0) => f.debug_tuple("UnexpectedEndOfInput").field(arg0).finish(),
            Self::Serde(arg0) => f.debug_tuple("Serde").field(arg0).finish(),
            Self::SocketError(arg0) => f.debug_tuple("SocketError").field(arg0).finish(),
            // Self::SocketError(arg0) => f.debug_tuple("SocketError").field(arg0).finish(),
            // _ => panic!("No error Message")
        }
    }
}
