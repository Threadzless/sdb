use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::RecordId;
use crate::prelude::*;

/// A RecordLink, which can contain either a [RecordId], or a Record.
///
/// This makes it both easy and fun to use a single record definition
/// schema that will work both when using **FETCH** clauses and not using them.
#[derive(Serialize)]
// #[serde(untagged)]
pub enum RecordLink<T: SurrealRecord = AnyRecord> {
    Record(Box<T>),
    Link(RecordId),
}

impl<'de, T: SurrealRecord> Deserialize<'de> for RecordLink<T> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let val = Value::deserialize(deserializer)?;

        match &val {
            Value::String(s) => Ok(Self::Link(RecordId::parse(s).unwrap())),
            _ => match serde_json::from_value(val) {
                Ok(thing) => Ok(Self::Record(Box::new(thing))),
                Err(err) => {
                    panic!("{err:?}")
                }
            },
        }
    }
}

impl<T: SurrealRecord> RecordLink<T> {
    /// Retrieves the [RecordId] of this field, regardless of if its'
    /// a [RecordId] or a struct representing the record the [RecordId]
    /// would point to.
    pub fn get_id(&self) -> RecordId {
        match self {
            Self::Link(l) => l.clone(),
            Self::Record(r) => (*r).id(),
        }
    }

    pub fn unwrap(&self) -> &T {
        self.record().unwrap()
    }

    pub fn record(&self) -> Option<&T> {
        match &self {
            RecordLink::Record(r) => Some(r),
            RecordLink::Link(_) => None,
        }
    }

    pub fn record_mut(&mut self) -> Option<&mut T> {
        match self {
            RecordLink::Record(r) => Some(r),
            RecordLink::Link(_) => None,
        }
    }

    // /// Returns a copy of inner record serialized as a [serde_json::Value], or
    // /// [serde_json::Value::Null] if **FETCH** was not used on this fields
    // pub fn value(&self) -> Result<Value, serde_json::Error> {
    //     match self {
    //         Self::Link(_l) => Ok(Value::Null),
    //         Self::Record(r) => serde_json::to_value(r.clone()),
    //     }
    //     // self.record().unwrap()
    // }
}
