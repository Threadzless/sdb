#![warn(missing_docs)]

/*!
Surreal Data Base client
=========================

An unofficial official client for SurrealDb, with convienience macros and compile time syntax checking.

*/

#[cfg(test)]
#[allow(unused)]
use serde::Serialize;
#[cfg(test)]
#[allow(unused)]
use serde_json::Value;

// pub use sdb_macros::*;
pub use sdb_base::*;

// documentation links
#[allow(unused_imports)]
use sdb_base as sdb;
#[allow(unused_imports)]
use sdb_base::prelude::*;

pub use sdb_macros::query;

pub use sdb_macros::SurrealRecord;

/// This is the only thing you need to import.
pub mod prelude {
    pub use sdb_base::prelude::*;

    pub use sdb_macros::*;

    pub use crate::query;
}

// #[cfg(doctest)]
// pub use sdb_base::example;
