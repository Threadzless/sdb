use serde::Deserialize;

use crate::record_id::RecordId;


/// Anything which acts as a record in SurrealDB.
/// 
/// ```rust
/// use sdb::prelude::*;
/// 
/// #[derive(Serialize, Deserialize, SurrealRecord)]
/// struct Book {
///     pub id: RecordId,
///     pub title: String,
///     pub blurb: Option<String>,
///     pub tags: Vec<String>,
///     pub author: RecordLink<Author>,
///     pub publisher: RecordLink<Publisher>,
/// }
/// 
/// #[derive(Serialize, Deserialize, SurrealRecord)]
/// struct Author {
///     pub id: RecordId,
///     pub name: String,
/// }
/// 
/// #[derive(Serialize, Deserialize, SurrealRecord)]
/// struct Publisher {
///     pub id: RecordId,
///     pub name: String,
/// }
/// 
/// ```
pub trait SurrealRecord: for<'de> Deserialize<'de> {
    fn id(&self) -> RecordId;
    fn table_name(&self) -> String {
        self.id().table()
    }
}

/// Anything which can be stored in SurrealDb without conversions.
/// 
/// This is not exactly the same as something implementing [`serde::Serialize`],
/// because record id's are not parsed the same way strings are in SurrealQL.
pub trait ToSurrealQL {
    fn to_sql(&self) -> String;
}



/// Set a field on a record to null.
pub struct SdbNull;
impl ToSurrealQL for SdbNull {
    fn to_sql(&self) -> String {
        "null".to_string()
    }
}

/// Unset a field 
pub struct SdbUnset;
impl ToSurrealQL for SdbUnset {
    fn to_sql(&self) -> String {
        "unset".to_string()
    }
}

impl ToSurrealQL for &str {
    fn to_sql(&self) -> String {
        format!("{self:?}")
    }
}

macro_rules! impl_to_sql {
    ($T: ty) => {
        impl ToSurrealQL for $T {
            fn to_sql(&self) -> String {
                format!("{self}")
            }
        }  
    };
}

impl ToSurrealQL for String {
    fn to_sql(&self) -> String {
        format!("{self:?}")
    }
}

impl<T: ToSurrealQL> ToSurrealQL for Vec<T> {
    fn to_sql(&self) -> String {
        let sql = self.iter()
            .map(|t| t.to_sql())
            .collect::<Vec<String>>()
            .join(", ");
        format!("[ {sql} ]")
    }
}


impl<T: ToSurrealQL> ToSurrealQL for Option<T> {
    fn to_sql(&self) -> String {
        match self {
            Some( v ) => v.to_sql(),
            None => format!("null"),
        }
    }
}


impl<T: ToSurrealQL> ToSurrealQL for &T {
    fn to_sql(&self) -> String {
        (*self).to_sql()
    }
}

impl ToSurrealQL for char {
    fn to_sql(&self) -> String {
        format!("{self:?}")
    }
}


impl_to_sql!(bool);
impl_to_sql!(i8);
impl_to_sql!(u8);

impl_to_sql!(i16);
impl_to_sql!(u16);

impl_to_sql!(i32);
impl_to_sql!(u32);

impl_to_sql!(i64);
impl_to_sql!(u64);

impl_to_sql!(i128);
impl_to_sql!(u128);


