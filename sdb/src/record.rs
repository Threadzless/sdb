use serde_json::{Value, Map};
use serde::{Serialize, Deserialize};

mod any;
mod id;
mod link;

pub use any::*;
pub use id::*;
pub use link::*;

/// Anything which acts as a record in SurrealDB. Records can contain 
/// links to other record types, which can be refer
///
/// This trait also implements a derive macro
///
/// ## Examples
/// ```rust
/// use sdb::prelude::*;
/// use serde::{Serialize, Deserialize};
/// 
/// #[derive(Serialize, Deserialize, SurrealRecord)]
/// #[table("books")]
/// struct Book {
///     pub id: RecordId,
///     pub title: String,
///     pub blurb: Option<String>,
///     pub tags: Vec<String>,
///     // RecordLink<T> is an enum RecordId or the contents of that record. 
///     pub author: RecordLink<Author>,
///     pub publisher: RecordLink<Publisher>,
/// }
///
/// #[derive(Serialize, Deserialize, SurrealRecord)]
/// #[table("authors")]
/// struct Author {
///     pub id: RecordId,
///     pub name: String,
/// }
///
/// #[derive(Serialize, Deserialize, SurrealRecord)]
/// #[table("publishers")]
/// struct Publisher {
///     pub id: RecordId,
///     pub name: String,
/// }
/// ```
pub trait SurrealRecord: for<'de> Deserialize<'de> + Serialize {
    fn id(&self) -> &RecordId;
    fn table_name(&self) -> String;

    fn link(&self) -> RecordLink<Self> {
        RecordLink::Link( self.id().clone() )
    }

    fn record_fields(&self) -> Map<String, Value> {
        match serde_json::to_value( self ) {
            Ok( parsed ) if parsed.is_object() => {
                let Value::Object( obj ) = parsed else { panic!() };
                obj
            },
            _ => unreachable!(),
        }
    }
}