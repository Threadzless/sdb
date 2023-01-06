use serde::Serialize;
use serde_json::{Map, Value};

use crate::{credentials::Credentials, error::*, server_info::ServerInfo};

#[derive(Serialize)]
pub struct SurrealRequest {
    pub id: String,
    pub method: String,
    pub params: Vec<Value>,
}

impl SurrealRequest {
    pub fn new(id: String, method: impl ToString, params: Vec<impl Into<Value>>) -> Self {
        Self {
            id,
            method: method.to_string(),
            params: params.into_iter().map(|s| s.into()).collect::<Vec<Value>>(),
        }
    }

    pub fn query(sql: impl ToString) -> Self {
        let id = String::from("12345");
        let req = Self::new(id, "query", vec![sql.to_string()]);

        req
    }

    pub fn use_ns_db(ns: &str, db: &str) -> Self {
        let id = String::from("12345");
        let req = Self::new(id, "use", vec![ns.to_string(), db.to_string()]);

        req
    }

    pub fn stringify(&self) -> SdbResult<String> {
        serde_json::to_string(self).map_err(|e| SdbError::Serde(e))
    }

    pub fn new_auth(info: &ServerInfo) -> Self {
        let id = String::from("12345");
        let mut vals = Map::new();
        // vals.insert("ns".to_string(), Value::String( info.namespace.clone() ));
        // vals.insert("db".to_string(), Value::String( info.database.clone() ));

        match &info.auth {
            Some(Credentials::Basic { user, pass }) => {
                vals.insert("user".to_string(), Value::String(user.clone()));
                vals.insert("pass".to_string(), Value::String(pass.clone()));
            }
            _ => unimplemented!(
                "That auth method is not currently compatible with socket connections"
            ),
        }

        Self::new(id, "signin", vec![Value::Object(vals)])
        // todo!()
    }
}
