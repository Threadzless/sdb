use serde::Deserialize;

mod any;
mod id;
mod link;

pub use any::*;
pub use id::*;
pub use link::*;

/// Anything which acts as a record in SurrealDB.
///
/// This trait also implements a derive macro
///
/// ## Examples
/// ```rust
/// # use sdb_macros::*;
/// # use sdb_base::prelude::*;
/// # use sdb_base as sdb;
/// # use serde::{Serialize, Deserialize};
/// #
/// #[derive(Serialize, Deserialize, SurrealRecord)]
/// #[table("books")]
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
pub trait SurrealRecord: for<'de> Deserialize<'de> {
    fn id(&self) -> RecordId;
    fn table_name(&self) -> String {
        self.id().table()
    }
}

// /// Anything which can be stored in SurrealDb without conversions.
// ///
// /// This is not exactly the same as something implementing [`serde::Serialize`],
// /// because [`RecordId`]s are not parsed the same way strings are in SurrealQL.
// ///
// /// This is already implemented for the following:
// /// - **Strings** (`String`, `&str`, 'char`)
// /// - **Primitives** (`bool`, `u8`, `i16`, `u32`, `i32`, `f32`, ect...)
// /// - **Arrays** (`&[T]`, `Vec<T>`)
// /// - [`serde_json::Value`]
// pub trait ToSurrealQL {
//     fn to_sql(&self) -> String;
// }

// // - - - Numbers - - - //

// macro_rules! impl_to_sql {
//     ($T: ty) => {
//         impl ToSurrealQL for $T {
//             fn to_sql(&self) -> String {
//                 format!("{self}")
//             }
//         }
//     };
// }

// impl_to_sql!(bool);
// impl_to_sql!(i8);
// impl_to_sql!(u8);

// impl_to_sql!(i16);
// impl_to_sql!(u16);

// impl_to_sql!(i32);
// impl_to_sql!(u32);

// impl_to_sql!(i64);
// impl_to_sql!(u64);

// impl_to_sql!(i128);
// impl_to_sql!(u128);

// // - - - Strings - - - //

// impl ToSurrealQL for String {
//     fn to_sql(&self) -> String {
//         format!("{self:?}")
//     }
// }

// impl ToSurrealQL for char {
//     fn to_sql(&self) -> String {
//         format!("{self:?}")
//     }
// }

// impl ToSurrealQL for &str {
//     fn to_sql(&self) -> String {
//         format!("{self:?}")
//     }
// }

// // - - - Arrays and options - - - //

// impl<T: ToSurrealQL> ToSurrealQL for Vec<T> {
//     fn to_sql(&self) -> String {
//         let sql = self
//             .iter()
//             .map(|t| t.to_sql())
//             .collect::<Vec<String>>()
//             .join(", ");
//         format!("[ {sql} ]")
//     }
// }

// impl<T: ToSurrealQL> ToSurrealQL for Option<T> {
//     fn to_sql(&self) -> String {
//         match self {
//             Some(v) => v.to_sql(),
//             None => format!("null"),
//         }
//     }
// }

// impl ToSurrealQL for &serde_json::Value {
//     fn to_sql(&self) -> String {
//         match serde_json::to_string( self ) {
//             Ok( val ) => val,
//             Err(err) => format!("Serializing to succeed:\n{:?}\n\n{:?}\n", self, err),
//         }
//     }
// }

// impl<T: ToSurrealQL> ToSurrealQL for &[ T ] {
//     fn to_sql(&self) -> String {
//         let sql = self
//             .iter()
//             .map(|t| t.to_sql())
//             .collect::<Vec<String>>()
//             .join(", ");
//         format!("[ {sql} ]")
//     }
// }

// // impl<T: ToSurrealQL> ToSurrealQL for &T {
// //     fn to_sql(&self) -> String {
// //         (*self).to_sql()
// //     }
// // }

// //

// //

// //

// /// A null value in a surrealDB transaction
// pub struct SdbNull;

// impl ToSurrealQL for SdbNull {
//     fn to_sql(&self) -> String {
//         "null".to_string()
//     }
// }

// /// Unset a field
// pub struct SdbUnset;

// impl ToSurrealQL for SdbUnset {
//     fn to_sql(&self) -> String {
//         "unset".to_string()
//     }
// }
