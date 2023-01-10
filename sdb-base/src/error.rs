use std::fmt::{Formatter, Debug, Result as FmtResult, Display};
use crate::client::SurrealResponseError;



pub type SdbResult<T> = Result<T, SdbError>;


// TODO: improve error system like a lot
#[derive(Debug)]
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
        message: SurrealResponseError
    },

    InvalidHostString {
        found: String,
    },

    QueryResultParseFailure {
        query: String,
        target_type: String,
        serde_err: serde_json::Error,
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
    HttpNetowrkError( reqwest::Error ),

    #[cfg(all(feature = "ws", not(target_family = "wasm")))]
    WebsocketNetworkError( websockets::WebSocketError ),

    // - - - WASM only - - - //

    #[cfg(all(feature = "http", target_family = "wasm"))]
    HttpNetowrkError( gloo_net::Error ),

    #[cfg(all(feature = "ws", target_family = "wasm"))]
    WebsocketNetworkError( gloo_net::websocket::WebSocketError ),
}



impl std::error::Error for SdbError { }

impl Display for SdbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(self, f)
    }
}

