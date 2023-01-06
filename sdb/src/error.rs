use std::{any::type_name, sync::PoisonError};
use std::fmt::{Formatter, Debug, Result as FmtResult, Display};
use reqwest::StatusCode;
use thiserror::Error as ThisError;
use serde_json::Error;

use crate::client::{SurrealResponse, SurrealResponseError};



pub type SdbResult<T> = Result<T, SdbError>;


// TODO: improve error system like a lot

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
}

impl std::error::Error for SdbError { }

impl Display for SdbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(self, f)
    }
}

impl Debug for SdbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::UnableToParseAsRecordId { input } => f.debug_struct("UnableToParseAsRecordId").field("input", input).finish(),
            Self::QuerySyntaxError { query, message } => f.debug_struct("QuerySyntaxError").field("query", query).field("message", message).finish(),
            Self::InvalidHostString { found } => f.debug_struct("InvalidHostString").field("found", found).finish(),
            Self::QueryResultParseFailure { query, target_type, serde_err } => f.debug_struct("QueryResultParseFailure").field("query", query).field("target_type", target_type).field("serde_err", serde_err).finish(),
            Self::ZeroQueryResults { query } => f.debug_struct("ZeroQueryResults").field("query", query).finish(),
            Self::NetworkTimeout => write!(f, "NetworkTimeout"),
        }
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