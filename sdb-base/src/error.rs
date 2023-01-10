use crate::{client::SurrealResponseError, reply::QueryReply};
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

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

impl std::error::Error for SdbError {}

impl Display for SdbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(self, f)
    }
}

impl SdbError {
    #[inline]
    pub(crate) fn parse_failure<T>(reply: &QueryReply, err: serde_json::Error) -> Self {
        SdbError::QueryResultParseFailure {
            query: reply.query(),
            target_type: core::any::type_name::<T>().to_string(),
            serde_err: err,
        }
    }
}
