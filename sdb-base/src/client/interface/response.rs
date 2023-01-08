use serde::Deserialize;

use crate::reply::QueryReply;

#[derive(Clone, Debug, Deserialize)]
pub struct SurrealResponseError {
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
        error: SurrealResponseError,
    },
}

type SurrealResponseResult = Result< Option<Vec<QueryReply>>, SurrealResponseError >;

impl Into<SurrealResponseResult> for SurrealResponse {
    fn into(self) -> SurrealResponseResult {
        match self {
            SurrealResponse::Result { id, result } => {
                Ok( result )
            },
            SurrealResponse::Error { id, error } => {
                Err( error )
            },
        }
    }
}





impl SurrealResponse {
    pub fn id(&self) -> String {
        match self {
            SurrealResponse::Result { id, .. } => id.clone(),
            SurrealResponse::Error { id, .. } => id.clone(),
        }
    }

    pub fn check_id(&self, compare: &str) -> bool {
        match self {
            SurrealResponse::Result { id, .. } => id.eq(compare),
            SurrealResponse::Error { id, .. } => id.eq(compare),
        }
    }
}
