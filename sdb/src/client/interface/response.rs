use serde::Deserialize;

use crate::reply::QueryReply;

#[derive(Clone, Debug, Deserialize)]
pub struct SocketResponseError {
    pub code: isize,
    pub message: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum SurrealResponse {
    Result {
        id: String,
        result: Option<Vec<QueryReply>>,
    },
    Error {
        id: String,
        error: SocketResponseError,
    },
}

impl SurrealResponse {
    pub fn check_id(&self, compare: &str) -> bool {
        match self {
            SurrealResponse::Result { id, .. } => id.starts_with(compare),
            SurrealResponse::Error { id, .. } => id.starts_with(compare),
        }
    }
}
