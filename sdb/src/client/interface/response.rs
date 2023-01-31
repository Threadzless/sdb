use ::serde::Deserialize;

use crate::{
    reply::StatementResult,
    client::interface::SurrealRequest,
};

#[derive(Clone, Debug, Deserialize)]
pub struct SurrealResponseError {
    pub code: isize,
    pub message: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum SurrealResponse {
    Error {
        id: u32,
        error: SurrealResponseError,
    },
    Result {
        id: u32,
        result: Option<Vec<StatementResult>>,
    },
}

unsafe impl Send for SurrealResponse {}
unsafe impl Sync for SurrealResponse {}

type SurrealResponseResult = Result<Option<Vec<StatementResult>>, SurrealResponseError>;

impl From<SurrealResponse> for SurrealResponseResult {
    fn from(val: SurrealResponse) -> SurrealResponseResult {
        match val {
            SurrealResponse::Result { result, .. } => Ok(result),
            SurrealResponse::Error { error, .. } => Err(error),
        }
    }
}

impl SurrealResponse {
    pub fn id(&self) -> u32 {
        match self {
            SurrealResponse::Result { id, .. } => *id,
            SurrealResponse::Error { id, .. } => *id,
        }
    }

    pub fn check_id(&self, compare: u32) -> bool {
        match self {
            SurrealResponse::Result { id, .. } => *id == compare,
            SurrealResponse::Error { id, .. } => *id == compare,
        }
    }

    pub fn is_for(&self, request: &SurrealRequest) -> bool {
        let req_id = request.id;
        match self {
            SurrealResponse::Result { id, .. } => *id == req_id,
            SurrealResponse::Error { id, .. } => *id == req_id,
        }
    }
}
