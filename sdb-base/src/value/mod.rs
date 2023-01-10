use std::collections::HashMap;

use crate::prelude::RecordId;

pub enum SurrealValue {
    Null,
    String(String),
    Object(HashMap<String, SurrealValue>),
    Array(Vec<SurrealValue>),
    RecordId(RecordId),
}
