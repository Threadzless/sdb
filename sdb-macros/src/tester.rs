// use proc_macro::Diagnostic;
use proc_macro_error::Diagnostic;
use serde_json::Value;
use syn::LitStr;

use crate::parts::TransactionParse;

mod local;
mod message;
#[cfg(feature = "query-test")]
mod remote;



// pub struct TestState<'a> {
//     // pub(crate) literal: &'a LitStr,
//     pub(crate) trans: &'a TransactionParse,
//     pub(crate) vars: Vec<(String, usize)>,
//     pub(crate) queries: Vec<(&'a LitStr, String)>,
// }

// impl<'a> TestState<'a> {
//     pub fn new( trans: &'a TransactionParse ) -> Self {
//         Self {
//             trans,
//             vars: trans.arg_vars(),
//             queries: trans.full_queries(),
//         }
//     }
// }


// pub struct QueryState<


pub fn check_syntax( trans: &TransactionParse ) -> Result<(), Diagnostic>{

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

/*
const DB_ACCESS_FAILED: &str = r#"Verify your connection string is set correctly, in .cargo/config.toml:
[env]
SURREAL_URL = "ws://test_user:test_pass@127.0.0.1/example/demo"

Or disable the query-test feature on the sdb crate
"#;

const ENV_VAR_NOT_SET: &str = r#"You have the `query-test` feature enabled, but the `SURREAL_URL` enviroment 
variables is not set, so queries cannot be test-run. To set `SURREAL_URL` at a project level, 
create a this file in your project directory:
        > .cargo/config.toml
        [env]
        # same as connection string: http://test_user:test_pass@127.0.0.1:8000/example/demo 
        SURREAL_USER = "test_user"
        SURREAL_PASS = "test_pass"
        SURREAL_HOST = "127.0.0.1:8000"
        SURREAL_NS   = "example"
        SURREAL_DB   = "demo"
"#;

/// Attempt to execute the query on
pub(crate) fn live_query_test(full_sql: String) {

    let Ok( mut req ) = build_request() else {
        emit_call_site_warning!( "`SURREAL_URL` not a valid connection string";
            help = ENV_VAR_NOT_SET
        );
        return;
    };

    req = req.body(format!("BEGIN;\n{full_sql};\nCANCEL;"));

    let res = match req.send() {
        Ok(res) => res,
        Err(err) => {
            return emit_call_site_warning!(
                "Unable to contact SurrealDB to verify query syntax";
                help = DB_ACCESS_FAILED;
                info = "Error details: {:?}", err;
            );
        }
    };

    let arr = match res.json::<Value>() {
        Ok(Value::Array(reply)) => reply,
        Ok(v) => {
            emit_call_site_error!(
                "Invalid query response Ok";
                help = "\n{:?}\n\n", v;
            );
            return;
        },
        Err(e) => {
            emit_call_site_error!(
                "Invalid query response Err";
                help = "\n{:?}\n\n", e;
            );
            return;
        }
    };
    // let Ok( reply ) = res.json::<Value>() else {
    //     panic!("2")
    // };
    // let Value::Array( arr ) = reply else { panic!("3") };
    // let arr = match res.json::<Value>() {
    //     Ok(reply) => reply,
    //     Err(e) => {
    //         emit_call_site_error!(
    //             "Invalid query response";
    //             help = "\n{:?}\n\n", e;
    //         );
    //         return;
    //     }
    // };


    for v in arr {
        match v {
            Value::Object( obj ) if
            let Some( detail ) = obj.get("detail") &&
            let Some( detail ) = detail.as_str() &&
            detail.eq("The query was not executed due to a cancelled transaction") => { },
            _ => {
                return emit_call_site_warning!("Query verification was inconclusive.")
            }
        }
    }
}

fn build_request() -> Result<RequestBuilder, VarError> {
    let host = env::var("SURREAL_HOST")?;
    let ns = env::var("SURREAL_NS")?;
    let db = env::var("SURREAL_DB")?;

    let client = Client::new();
    let url = format!("http://{host}/sql");
    let mut post = client
        .post(url)
        .header("Accept", "application/json")
        .header("NS", ns)
        .header("DB", db);

    match (env::var("SURREAL_USER"), env::var("SURREAL_PASS")) {
        (Ok(user), Ok(pass)) => {
            post = post.basic_auth(user, Some(pass));
        }
        (Ok(user), Err(_)) => {
            post = post.basic_auth(user, Option::<String>::None);
        }
        _ => {}
    };

    Ok(post)
}

// */