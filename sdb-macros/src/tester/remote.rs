use std::env::VarError;
use std::fmt::{Write, Error as FmtError};

use proc_macro_error::{Diagnostic, Level};
use proc_macro_error::{emit_warning, emit_error, emit_call_site_error};
use reqwest::blocking::*;
use serde_json::Value;
use syn::LitStr;

use crate::TransactionParse;

pub enum SurrealResponse {
    Completed( Vec<Value> ),
    SyntaxError( SyntaxError ),
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct SyntaxError {
    pub code: usize,
    pub description: String,
    pub details: String,
    pub information: String,
}

//

//

pub fn run_test( trans: &TransactionParse ) -> Result<(), Diagnostic> {
    // compile the complete SQL request
    let full_sql = match build_sql( trans ) {
        Ok(sql) => sql,
        Err(err) => {
            return Err(
                Diagnostic::new(Level::Error, r"Failed to construct the query string. ¯\_(ツ)_/¯".into())
                .help(ENV_VAR_NOT_SET.to_string())
                .note(format!("Original Error:\n{err:#?}"))
            )
        }
    };

    // create a `reqwest::RequestBuilder`
    let req = match prepare_request() {
        Ok(req) => req.body( full_sql.clone() ),
        Err(err) => {
            return Err(
                Diagnostic::new(Level::Error, "Testing Database is not setup".into())
                .help(ENV_VAR_NOT_SET.to_string())
                .note(format!("Original Error:\n{err:#?}"))
            )
        }
    };

    // Execute the request
    let res = match req.send() {
        Ok(res) => res,
        Err(err) => {
            return Err(
                Diagnostic::new(Level::Error, "Failed to contact the SurrealDB for query testing".into())
                .help(ENV_VAR_NOT_SET.to_string())
                .note(format!("Original Error:\n{err:#?}"))
            )
        }
    };

    // Process
    let reply = res.json::<Value>()
        .expect("Expected a parsable JSON response");

    // No errors from the server
    if reply.is_array() {
        return Ok( () )
    }

    let err = serde_json::from_value::<SyntaxError>(reply).expect("Unexpected response object");
        
    
    match handle_and_emit(&err, trans, full_sql) {
        Ok( true ) => { 
            // everything worked properly
            Ok( () )
        },
        Ok( false ) => {
            // there was a problem, but it couldn't be highlighted correctly
            Err(
                Diagnostic::new(Level::Error, "Surreal server responded with an error :(".into())
                    .help(format!("Server response:\n{err:?}")) 
            )
        },
        Err(d) => Err(d),
    }

    // TODO check if 
}

//

//

pub fn handle_and_emit(err: &SyntaxError, trans: &TransactionParse, sent_sql: String) -> Result<bool, Diagnostic> {
    match get_location(&err.information) {
        Some((line, col, Some(region))) => {
            let Some((lit, full_sql)) = get_litstr( line, trans ) else {
                panic!("get_litstr failed\n{sent_sql}\n{line}, {col}, {region}\n\n{err:#?}\n");
                return Ok(false)
            };
            let idx = match lit.value().find(&region) { 
                Some(idx) => idx,
                None if let Some(idx) = full_sql.find(&region) => idx,
                _ => return Ok(false)
            };
            let line_span = lit.span().unwrap();
            let mut l = proc_macro::Literal::string(&lit.value());
            l.set_span(line_span);
            let range = idx+1..(idx+region.len()+1);
            let highlight = l.subspan(range).unwrap_or( line_span );
            emit_error!(
                highlight, "Syntax error in SurrealQL";
                help = "\n > {}\n", full_sql;
                info = "Server Response:\n{:#?}\n", err;
            );
            return Ok(true)
        },

        Some((line, col, None)) => {
            let Some((lit, full_sql)) = get_litstr( line, trans ) else { 
                return Err(
                    Diagnostic::new(Level::Error, "Syntax error in SurrealQL".to_string())
                    .note(format!("Server Response:\n{:#?}\n", err))
                )
                //     .
                // emit_call_site_error!(
                //     "Syntax error in SurrealQL";
                //     info = "Server Response:\n{:#?}\n", err;
                // );
                // return true
            };
            emit_error!(
                lit, "Syntax error in SurrealQL";
                help = "\n > {}\n", full_sql;
                info = "Server Response:\n{:#?}\n", err;
            );
            return Ok(true)
        },

        _ => panic!("gggggg")
    }

    return Ok(false)
}

//

//

pub fn get_litstr<'a>( line: usize, trans: &'a TransactionParse ) -> Option<(&'a LitStr, String)> {
    let mut line = (line as isize - 1); // sub 1 for BEGIN statement
    if let Some( ref args ) = trans.args {
        for f in args.fields.iter() {
            match f {
                crate::parts::QueryArg::Expr(_) => line -= 2,
                crate::parts::QueryArg::Var(_) => line -= 1,
                crate::parts::QueryArg::Alias { name, _colon, expr } => {
                    line -= 2;   
                },
            }
        }
    }

    if line < 0 {
        panic!("Highlight out of range! {line}")
    }

    let start_line = line;

    for (lit, full_sql) in trans.full_queries() {
        let lines_in_query = full_sql.chars()
            .filter(|c| '\n'.eq(c) )
            .count() as isize;

        line -= lines_in_query + 1;

        if line <= 1 { 
            return Some((lit, full_sql))
        }
    }
    None
}

//

//

pub fn get_location( info: &str ) -> Option<(usize, usize, Option<String>)> {
    if ! info.starts_with("There was a problem with the database: Parse error on line") {
        return None;
    }

    let rest = &info[59..];
    let first_num_str = rest.chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>();

    let rest = &rest[5..];
    let second_num_str = rest.chars()
        .skip_while(|c| ! c.is_digit(10) )
        .take_while(char::is_ascii_digit)
        .collect::<String>();


    let Ok( line ) = first_num_str.parse::<usize>() else { return None };
    let Ok( col ) = second_num_str.parse::<usize>() else { return None };

    if let Some(idx) = rest.find("when parsing '") {
        let mut region = &rest[(idx+14)..(rest.len()-1)];

        match region.find(";--\n") {
            Some(idx) => {
                let reg = &region[..idx];
                Some(( line, col, Some( String::from(reg)) ))
            },
            _ => {
                Some(( line, col, Some( String::from(region)) ))
            }
        }

        // Some(( line, col, Some( String::from(region)) ))
    }
    else {
        Some(( line, col, None ))
    }
}

//

//

pub fn build_sql( trans: &TransactionParse ) -> Result<String, FmtError> {
    let mut out = String::from("BEGIN;\n");
    if let Some(ref args) = trans.args {
        for (idx, field) in args.fields.iter().enumerate() {
            writeln!(out, "LET ${idx} = 0;")?;
            match field {
                crate::parts::QueryArg::Var(ident) => {
                    writeln!(out, "LET ${} = 0;", ident.to_string())?;
                },
                _ => { }
            }
        }
    }

    use crate::parts::SdbStatement as SdbS;
    for (idx, stmt) in trans.lines.iter().enumerate() {
        match stmt {
            SdbS::Import { .. } => { },
            SdbS::Ignored { sql } |
            SdbS::ToVar { sql, .. } |
            SdbS::Parse { sql, .. } |
            SdbS::ParseTail { sql, .. } => {
                if sql.methods.len() > 0 {
                    writeln!(out, "{};--", sql.literal.value());
                    writeln!(out, "{};--", sql.complete_sql());
                }
                else {
                    writeln!(out, "{};--", sql.literal.value());
                    writeln!(out, "");
                }
            },
        }
    }

    write!(out, "CANCEL TRANSACTION")?;
    
    Ok(out)
}

//

//

pub fn prepare_request() -> Result<RequestBuilder, VarError> {
    use std::env::var;
    let host = var("SURREAL_HOST")?;
    let ns = var("SURREAL_NS")?;
    let db = var("SURREAL_DB")?;

    let client = Client::new();
    let url = format!("http://{host}/sql");
    let mut post = client
        .post(url)
        .header("Accept", "application/json")
        .header("NS", ns)
        .header("DB", db);

    match (var("SURREAL_USER"), var("SURREAL_PASS")) {
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

















const ENV_VAR_NOT_SET: &str = r#"Verify your connection string is set correctly in .cargo/config.toml:
[env]
SURREAL_USER = "test_user"
SURREAL_PASS = "test_pass"
SURREAL_HOST = "127.0.0.1:8000"
SURREAL_NS   = "example"
SURREAL_DB   = "demo"
"#;