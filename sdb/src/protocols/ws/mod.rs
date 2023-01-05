use serde::{Serialize, Deserialize};
use serde_json::{Value, Map};

use crate::{reply::QueryReply, error::{SdbError, SdbResult}, server_info::ServerInfo, credentials::Credentials};
pub(crate) use super::SdbProtocol;

#[cfg(target_family="wasm")]
mod ws_wasm;
#[cfg(target_family="wasm")]
pub use ws_wasm::*;

#[cfg(not(target_family="wasm"))]
mod ws_rest;
#[cfg(not(target_family="wasm"))]
pub use ws_rest::*;

#[derive(Serialize)]
pub struct SocketRequest {
    pub id: String,
    pub method: String,
    pub params: Vec<Value>,
}

impl SocketRequest {
    pub fn new(id: String, method: impl ToString, params: Vec<impl Into<Value>> ) -> Self {
        Self {
            id,
            method: method.to_string(),
            params: params.into_iter()
                .map( |s| s.into() )
                .collect::<Vec<Value>>(),
        }
    }

    pub fn query( sql: impl ToString ) -> Self {
        let id = String::from("12345"); 
        let req = Self::new(id, "query", vec![ sql.to_string() ]);

        req
    }

    pub fn use_ns_db( ns: &str, db: &str ) -> Self {
        let id = String::from("12345"); 
        let req = Self::new(id, "use", vec![ ns.to_string(), db.to_string() ]);

        req
    }

    pub fn stringify(&self) -> SdbResult<String> {
        serde_json::to_string(self)
            .map_err(|e| SdbError::Serde( e ))
    }

    pub fn new_auth( info: &ServerInfo ) -> Self {
        let id = String::from("12345"); 
        let mut vals = Map::new();
        // vals.insert("ns".to_string(), Value::String( info.namespace.clone() ));
        // vals.insert("db".to_string(), Value::String( info.database.clone() ));

        match &info.auth {
            Some( Credentials::Basic { user, pass } ) => {
                vals.insert("user".to_string(), Value::String( user.clone() ));
                vals.insert("pass".to_string(), Value::String( pass.clone() ));
            },
            _ => unimplemented!("That auth method is not currently compatible with socket connections")
        }

        Self::new(id, "signin", vec![ Value::Object(vals) ])
        // todo!()
    }
}


#[derive(Deserialize)]
pub struct SocketResponseError {
    pub code: isize,
    pub message: String,
}


#[derive(Deserialize)]
#[serde(untagged)]
pub enum SocketResponse {
    Result {
        id: String,
        result: Vec<QueryReply>,
    },
    Error {
        id: String,
        error: SocketResponseError
    },
}

impl SocketResponse {
    pub fn check_id(&self, compare: &str) -> bool {
        match self {
            SocketResponse::Result { id, .. } => id.starts_with( compare ),
            SocketResponse::Error { id, .. } => id.starts_with( compare ),
        }
    }
}
