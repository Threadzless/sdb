use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{from_value, Value};
use std::ops::{Deref, DerefMut};

use crate::{record::SurrealRecord, record_id::RecordId};

/// A Generic record which can hold any value. It's basically
/// just [`Value`](serde_json::Value) but it implements
/// [`SurrealRecord`]
#[derive(Clone, Debug, PartialEq)]
pub struct AnyRecord {
    val: Value,
}

impl Serialize for AnyRecord {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.val.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AnyRecord {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self {
            val: <Value as Deserialize>::deserialize(deserializer)?,
        })
    }
}

impl SurrealRecord for AnyRecord {
    fn id(&self) -> RecordId {
        if let Value::Object( o ) = &self.val
        && let Some( id ) = o.get("id") {
            from_value(id.clone()).unwrap()
        }
        else {
            unimplemented!()
        }
    }
}

impl Deref for AnyRecord {
    type Target = Value;
    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

impl DerefMut for AnyRecord {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}
