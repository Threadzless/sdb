use serde::{de::*, *};
use std::fmt::{Formatter, Result as FmtResult, Display};

use crate::error::{SdbError, SdbResult};

use super::{RecordLink, SurrealRecord};

const PLACEHOLDER_KEY: &str = "\u{0}";

/// The `id` field of all SurrealDB records, and a
///
#[derive(Clone, Debug, PartialEq)]
pub struct RecordId {
    table: String,
    key: String,
}

impl RecordId {
    /// TODO: check for spaces, too long, invalid characters, etc.
    pub fn parse(text: impl ToString) -> SdbResult<Self> {
        let text = text.to_string();
        match text.split_once(':') {
            Some((table, key)) => Ok(Self::new(table, key)),
            None => Err(SdbError::UnableToParseAsRecordId {
                input: text.clone(),
            }),
        }
    }

    pub fn new(table: impl ToString, key: impl ToString) -> Self {
        Self {
            table: table.to_string(),
            key: key.to_string(),
        }
    }

    pub fn id(&self) -> RecordId {
        self.clone()
    }

    pub fn key(&self) -> String {
        self.key.clone()
    }

    pub fn table(&self) -> String {
        self.table.clone()
    }

    pub(crate) fn is_placeholder(&self) -> bool {
        self.key.eq(PLACEHOLDER_KEY)
    }

    /// Used for creating records
    pub fn placeholder(table_name: &str) -> Self {
        Self {
            table: table_name.to_string(),
            key: PLACEHOLDER_KEY.to_string()
        }
    }
}

impl<T: SurrealRecord> Into<RecordLink<T>> for RecordId {
    fn into(self) -> RecordLink<T> {
        todo!()
    }
}

// impl ToSurrealQL for RecordId {
//     fn to_sql(&self) -> String {
//         format!("type::thing( {:?}, {:?} )", self.table, self.key)
//     }
// }

impl Display for RecordId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}:`{}`", self.table, self.key)
    }
}

impl Serialize for RecordId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for RecordId {
    fn deserialize<D: Deserializer<'de>>(des: D) -> Result<Self, D::Error> {
        des.deserialize_string(RecordIdVisitor)
    }
}

struct RecordIdVisitor;

impl<'de> Visitor<'de> for RecordIdVisitor {
    type Value = RecordId;

    fn expecting(&self, _f: &mut Formatter) -> FmtResult {
        Ok(())
    }

    fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
        match RecordId::parse(v) {
            Ok(id) => Ok(id),
            Err(_) => Err(E::custom("Cannot parse input as a RecordId")),
        }
    }

    fn visit_string<E: Error>(self, v: String) -> Result<Self::Value, E> {
        match RecordId::parse(v) {
            Ok(id) => Ok(id),
            Err(_) => Err(E::custom("Cannot parse input as a RecordId")),
        }
    }
}
