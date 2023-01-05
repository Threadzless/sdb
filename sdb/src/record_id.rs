use serde::{de::*, *};
use std::fmt::{Formatter, Result as FmtResult};

use crate::{
    error::{SdbResult, SdbError},
    record::ToSurrealQL,
};



#[derive(Clone, Debug, PartialEq)]
pub struct RecordId {
    table: String,
    key: String,
}

impl RecordId {
    /// TODO: check for spaces, too long, invalid characters, etc.
    pub fn parse(text: impl ToString) -> SdbResult<Self> {
        let text = text.to_string();
        let (table, key) = text.split_once(":")
            .ok_or( SdbError::UnableToParseAsRecordId( text.clone() ) )?;
        Ok(Self::new(table, key))
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
}

impl ToSurrealQL for RecordId {
    fn to_sql(&self) -> String {
        format!("type::thing( {}:`{}` )", self.table, self.key)
    }
}

impl ToString for RecordId {
    fn to_string(&self) -> String {
        format!("{}:`{}`", self.table, self.key)
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
        match RecordId::parse(&v) {
            Ok(id) => Ok(id),
            Err(_) => Err(E::custom("Cannot parse input as a RecordId")),
        }
    }
}
