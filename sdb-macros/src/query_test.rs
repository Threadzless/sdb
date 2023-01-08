#![allow(unused_imports)]

use std::env::var;
use proc_macro2::Span;
use proc_macro_error::emit_warning;
use crate::parts::TransFunc;

#[cfg(feature = "query-test")]
use sdb_base::{
    prelude::SurrealClient,
    reply::{query, TransactionReply}
};

#[cfg(not(feature = "query-test"))]
pub(crate) fn query_check( _func: &TransFunc ) { }



#[cfg(feature = "query-test")]
pub(crate) fn query_check( func: &TransFunc ) {
    use sdb_base::transaction::TransactionBuilder;
    use quote::ToTokens;
    use tokio::{task::{spawn_blocking, block_in_place}, spawn, runtime::Handle};

    use crate::parts::{TransFunc, LetQueryLine, LetQueryInput};

    if let Ok( no_test ) = var("SURREAL_NO_TEST") && no_test.ne("1") {
        return;
    }

    let Ok( surreal_url ) = var("SURREAL_URL") else {
        let span = Span::mixed_site();
        return emit_warning!( span, "`SURREAL_URL` not set";
            help = r#"You have the `test-query` feature enabled on this crate, but the `SURREAL_URL` enviroment 
variables is not set, so queries cannot be test-run. To set `SURREAL_URL` at a project level, 
create a this file in your project directory:
        > .cargo/config.toml
        [env]
        SURREAL_URL = "ws://test_user:test_pass@127.0.0.1/example/demo"
"#
        )
    };

    let mut steps = Vec::new();
    for line in func.iter_lines() {
        match line {
            crate::QueryLine::Raw( raw ) => {
                steps.push( TestStep::Raw { query: raw.sql.value() } )
            },
            crate::QueryLine::Let( lql ) if 
            let LetQueryInput::Query( query ) = &lql.input => {
                steps.push( TestStep::LetQuery {
                    var: lql.var.to_string(),
                    query: query.complete_sql()
                } )
            },
            crate::QueryLine::Select( select ) => {
                steps.push( TestStep::Select {
                    query: select.sql.complete_sql(),
                })
            },
            _ => continue,
        }
    }


    let site = Span::mixed_site();

    let Ok( run ) = tokio::runtime::Runtime::new() else {
        return emit_warning!(
            site, "Failed to access SurrealDB to verify query syntax";
            help = ""
        );
    };
    let handle = run.spawn( test_run(surreal_url, steps) );

    match run.block_on( handle ) {
        Ok( Ok( _reply ) ) => {
            // for line in func.iter_lines() {
            //     reply.
            // }
        },
        _ => {
            return emit_warning!(
                site, "Failed to access SurrealDB to check query";
                help = r#"Verify your connection string is set correctly, in .cargo/config.toml:
    [env]
    SURREAL_URL = "ws://test_user:test_pass@127.0.0.1/example/demo"

Or disable the query-test feature on the sdb crate
"#
            );
        }
    }
}

#[cfg(feature = "query-test")]
async fn test_run( url: String, steps: Vec<TestStep> ) -> Result<TransactionReply, sdb_base::error::SdbError> {
    use sdb_base::{prelude::SurrealClient, reply::{query, TransactionReply}};

    let client = SurrealClient::new( &url )
        .build()?;

    let mut trans = client.transaction();

    for step in steps {
        trans = match step {
            TestStep::Raw { query } => {
                trans.push(true, query)
            },
            TestStep::LetQuery { var, query } => {
                trans.push_var(var.as_str(), query)
            },
            TestStep::Select { query } => {
                trans.push(false, query)
            },
        }
    }

    trans.run().await
}


#[cfg(feature = "query-test")]
enum TestStep {
    Raw { 
        query: String
    },
    LetQuery {
        var: String,
        query: String,
    },
    Select {
        query: String
    }
}