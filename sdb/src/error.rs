use ::std::fmt::{*, Result as FmtResult};
use ::serde_json::Value;

use crate::client::SurrealResponseError;

pub type SdbResult<T> = std::result::Result<T, SdbError>;

// TODO: improve error system like a lot
pub enum SdbError {
    UnableToParseAsRecordId {
        input: String,
    },

    /// The server isn't a SurrealDB instance
    ServerNotSurreal {
        why: String,
    },

    QuerySyntaxError {
        query: String,
        message: SurrealResponseError,
    },

    /// Hoststring match this format (without spaces):
    /// ```html
    /// [ <protocol>:// ] [ <username> [ : <password> ] @ ] <url_with_port> / <namespace> / <database>
    /// ```
    ///
    /// ### Examples
    /// ```ini
    ///   ws://test_user:test_pass@127.0.0.1:8934/test/demo
    /// http://                    127.0.0.1:8000/example_ns/demo_db
    /// ```
    InvalidHostString {
        found: String,
    },

    QueryResultParseFailure {
        query: String,
        target_type: String,
        serde_err: serde_json::Error,
        value: Option<Value>,
    },

    ConnectionClosed {
        info: String,
        url: String,
    },

    /// Attempted to parse one result, but found none
    ZeroQueryResults {
        query: String,
    },

    /// The query included a **TIMEOUT** clause, and the alotted time was
    /// exceeded.
    QueryTimeout,

    /// The Server took too long to respond, so the transaction timed out.
    ///
    /// This was likely
    NetworkTimeout,

    /// Attempted to contact
    ConnectionRefused {
        url: String,
    },

    ///
    OversizedPayload,

    // Non-specific. Ideally all errors below will be converted into
    // one of the errors above instead of being passed.

    // - - - Non-WASM - - - //
    #[cfg(all(feature = "http", not(target_family = "wasm")))]
    HttpNetowrkError(reqwest::Error),

    #[cfg(all(feature = "ws", not(target_family = "wasm")))]
    WebsocketNetworkError(websockets::WebSocketError),

    // - - - WASM only - - - //
    #[cfg(all(feature = "http", target_family = "wasm"))]
    HttpNetowrkError(gloo_net::Error),

    #[cfg(all(feature = "ws", target_family = "wasm"))]
    WebsocketNetworkError(gloo_net::websocket::WebSocketError),
}

impl Debug for SdbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::QueryResultParseFailure { query, target_type, value: Some( v ), .. } => {
                write!(f, "Failed to parse value as {target_type}:\n\t{v:?}\n\n{query}\n")
            },
            Self::QueryResultParseFailure { query, target_type, value: None, .. } => {
                write!(f, "Failed to parse value as {target_type}\n\n{query}\n")
            },

            Self::UnableToParseAsRecordId { input } => f.debug_struct("UnableToParseAsRecordId").field("input", input).finish(),
            Self::ServerNotSurreal { why } => f.debug_struct("ServerNotSurreal").field("why", why).finish(),
            Self::QuerySyntaxError { query, message } => f.debug_struct("QuerySyntaxError").field("query", query).field("message", message).finish(),
            Self::InvalidHostString { found } => f.debug_struct("InvalidHostString").field("found", found).finish(),
            Self::ConnectionClosed { info, url } => f.debug_struct("ConnectionClosed").field("info", info).field("url", url).finish(),
            Self::ZeroQueryResults { query } => f.debug_struct("ZeroQueryResults").field("query", query).finish(),
            Self::QueryTimeout => write!(f, "QueryTimeout"),
            Self::NetworkTimeout => write!(f, "NetworkTimeout"),
            Self::ConnectionRefused { url } => f.debug_struct("ConnectionRefused").field("url", url).finish(),
            Self::OversizedPayload => write!(f, "OversizedPayload"),
            
            // x86 only
            #[cfg(all(feature = "http", not(target_family = "wasm")))]
            Self::HttpNetowrkError(arg0) => f.debug_tuple("HttpNetowrkError").field(arg0).finish(),
            #[cfg(all(feature = "http", not(target_family = "wasm")))]
            Self::WebsocketNetworkError(arg0) => f.debug_tuple("WebsocketNetworkError").field(arg0).finish(),

            // wasm only 
            #[cfg(all(feature = "http", target_family = "wasm"))]
            Self::HttpNetowrkError(arg0) => f.debug_tuple("HttpNetowrkError").field(arg0).finish(),
            #[cfg(all(feature = "http", target_family = "wasm"))]
            Self::WebsocketNetworkError(arg0) => f.debug_tuple("WebsocketNetworkError").field(arg0).finish(),
        }
    }
}

impl SdbError {
    #[inline]
    pub(crate) fn parse_failure<T>(reply: &crate::reply::StatementResult, err: serde_json::Error) -> Self {
        SdbError::QueryResultParseFailure {
            query: reply.query(),
            target_type: core::any::type_name::<T>().to_string(),
            serde_err: err,
            value: Some( reply.result.clone() )
        }
    }
}