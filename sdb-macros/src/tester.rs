use ::syn::LitStr;

use crate::parts::SdbArgs;


mod local;
#[cfg(feature = "query-test")]
mod remote;

/// Examines the syntax for out-of-order clauses, missed parenthesies, and other common
/// issues. If feature `query-test` is enabled, it will also execute the query in a 
/// rolled back transaction at compile time to ensure the syntax is perfect
pub(crate) fn check_syntax(
    vars: Vec<(String, usize)>,
    queries: Vec<(&LitStr, String)>,
    args: &Option<SdbArgs>,
) {

    match local::check(&vars, &queries) {
        Ok(_) => {},
        Err(e) => return e.emit(),
    }

    #[cfg(feature = "query-test")]
    match remote::run_test(&queries, args) {
        Ok(_) => {},
        Err(e) => return e.emit()
    }

}
