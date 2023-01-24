use serde::Serialize;
use serde_json::{Map, Value};

use crate::prelude::*;

#[derive(Serialize)]
pub struct SurrealRequest {
    pub id: u32,
    pub method: RequestMethod,
    pub params: Vec<Value>,
}

unsafe impl Send for SurrealRequest {}
unsafe impl Sync for SurrealRequest {}

impl SurrealRequest {
    fn new(method: RequestMethod, params: Vec<impl Into<Value>>) -> Self {
        Self {
            id: rand::random(),
            method,
            params: params.into_iter().map(|s| s.into()).collect::<Vec<Value>>(),
        }
    }

    pub fn query(sql: impl ToString) -> Self {
        // println!("\n{}\n", sql.to_string());
        Self::new(
            RequestMethod::Query,
            vec![sql.to_string()]
        )
    }

    pub fn use_ns_db(ns: &str, db: &str) -> Self {
        Self::new(
            RequestMethod::Use,
            vec![ns.to_string(), db.to_string()]
        )
    }

    pub fn stringify(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn new_auth(info: &ServerInfo) -> Self {
        let mut vals = Map::new();

        match &info.auth {
            Some(Credentials::Basic { user, pass }) => {
                vals.insert("user".to_string(), Value::String(user.clone()));
                vals.insert("pass".to_string(), Value::String(pass.clone()));
            }
            None => {}
            _ => unimplemented!(),
        }

        Self::new(
            RequestMethod::Signin,
            vec![Value::Object(vals)]
        )
    }
}
