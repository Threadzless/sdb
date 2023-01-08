use std::{any::type_name, sync::PoisonError};
use std::fmt::{Formatter, Debug, Result as FmtResult, Display};
use reqwest::StatusCode;
use thiserror::Error as ThisError;
use serde_json::Error;

use crate::client::{SurrealResponse, SurrealResponseError};



pub type SdbResult<T> = Result<T, SdbError>;


// TODO: improve error system like a lot
#[derive(Debug)]
pub enum SdbError {

    UnableToParseAsRecordId {
        input: String,
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

    /// Attempted to parse one result, but found none
    ZeroQueryResults {
        query: String,
    },

    NetworkTimeout,

    ConnectionRefused {
        url: String,
    },

    #[cfg(all(feature = "ws", not(target_family = "wasm")))]
    WebsocketNetworkError( websockets::WebSocketError ),
}

impl std::error::Error for SdbError { }

impl Display for SdbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(self, f)
    }
}

impl From<reqwest::Error> for SdbError {
    fn from(value: reqwest::Error) -> Self {
        if value.is_timeout() {
            SdbError::NetworkTimeout
        }
        else {
            panic!("{value:?}")
        }
    }
}

impl From<websockets::WebSocketError> for SdbError {

    fn from(value: websockets::WebSocketError) -> Self {
        SdbError::WebsocketNetworkError( value )
        // use websockets::{WebSocketError as WsErr, };
        // match value {
        //     WsErr::TcpConnectionError( err ) => {
        //         match err.kind() {
        //             std::io::ErrorKind::ConnectionRefused => {
        //                 SdbError::ConnectionRefused{
        //                     url: err.
        //                 }   
        //             },
        //             _ => SdbError::WebsocketNetworkError( value ),
        //         }
        //     },
        //     _ => SdbError::NetworkError,
        // }
    }
}