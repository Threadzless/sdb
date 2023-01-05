// pub mod error;
pub mod query;
pub mod transaction;

// pub use error::*;
pub use query::*;
pub use transaction::*;

// pub type SdbResult<T> = Result<T, SdbError>;

// /// A module containing the variant values of all Error enums
// /// in the sdb crate.
// ///
// /// Feel free to `use sdb::error_vals::*;` if your error handling get cluttered.
// pub mod error_vals {
//     pub use super::error::SdbError::*;
//     pub use super::error::SdbNetworkError::*;
// }
