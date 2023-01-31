use ::proc_macro_error::Diagnostic;

use crate::parts::QueryParse;

mod local;
#[cfg(feature = "query-test")]
mod remote;


pub(crate) fn check_syntax( trans: &QueryParse ) -> Result<(), Diagnostic>{

    match local::check(trans) {
        Ok(_) => {},
        Err(e) => {
            e.emit();
            return Ok( () )
        },
    }

    #[cfg(feature = "query-test")]
    remote::run_test(trans)?;

    Ok( () )
}
