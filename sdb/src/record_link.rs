use serde::{de::*, Deserialize, Serialize};
use serde_json::Value;
use std::{marker::PhantomData, str::FromStr};

use crate::{any_record::AnyRecord, error::SdbError, record::SurrealRecord, record_id::RecordId};

/// A RecordLink, which can contain either a [RecordId], or a Record.
///
/// This makes it both easy and fun to use a single record definition
/// schema that will work both when using **FETCH** clauses and not using them.
#[derive(Clone, PartialEq, Debug, Serialize)]
#[serde(untagged)]
pub enum RecordLink<T: SurrealRecord = AnyRecord> {
    Record(Box<T>),
    Link(RecordId),
}

impl<'de, T: SurrealRecord> Deserialize<'de> for RecordLink<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        string_or_struct(deserializer)
    }
}

impl<T: SurrealRecord> RecordLink<T>
where
    T: Serialize + for<'de2> Deserialize<'de2> + Sized,
{
    /// Retrieves the [RecordId] of this field, regardless of if its'
    /// a [RecordId] or a struct representing the record the [RecordId]
    /// would point to.
    pub fn get_id(&self) -> RecordId {
        match self {
            Self::Link(l) => l.clone(),
            Self::Record(r) => (*r).id(),
        }
    }

    pub fn record(&self) -> Option<&T> {
        match &self {
            RecordLink::Record(r) => Some(&r),
            RecordLink::Link(_) => None,
        }
    }

    pub fn record_mut(&mut self) -> Option<&mut T> {
        match self {
            RecordLink::Record(r) => Some(r),
            RecordLink::Link(_) => None,
        }
    }

    /// Returns a copy of inner record serialized as a [serde_json::Value], or
    /// [serde_json::Value::Null] if **FETCH** was not used on this fields
    pub fn value(&self) -> Result<Value, serde_json::Error> {
        match self {
            Self::Link(_l) => Ok(Value::Null),
            Self::Record(r) => serde_json::to_value(r.clone()),
        }
        // self.record().unwrap()
    }
}

impl<T: SurrealRecord> FromStr for RecordLink<T> {
    type Err = SdbError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        RecordId::parse(s).map(|id| Self::Link(id))
    }
}

/// Deserializes a field as either a string or a struct. This is for parsing
/// [RecordLink]s, and will likely not be needed by library users, but it must
/// be public for the compiler
pub fn string_or_struct<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr, //<Err = ()>,
    D: Deserializer<'de>,
{
    struct StringOrStruct<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for StringOrStruct<T>
    where
        T: Deserialize<'de> + FromStr, //<Err = ()>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E: Error>(self, value: &str) -> Result<T, E> {
            match FromStr::from_str(value) {
                Ok(s) => Ok(s),
                _ => panic!(),
            }
            // Ok(.unwrap())
        }

        fn visit_map<M: MapAccess<'de>>(self, map: M) -> Result<T, M::Error> {
            Deserialize::deserialize(value::MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(StringOrStruct(PhantomData))
}
