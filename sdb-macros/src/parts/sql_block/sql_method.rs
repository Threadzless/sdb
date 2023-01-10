use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

#[allow(unused_imports)]
use proc_macro_error::{emit_error, emit_warning};

use quote::ToTokens;

#[allow(unused_imports)]
use syn::{parse::*, punctuated::Punctuated, *};

const UNKNOWN_METHOD_HELP: &str = r#"Expected one of the following methods, or no method:
 - pluck
 - limit
 - shuffle
 - count
 - one
 - page
"#;

#[derive(Debug)]
pub(crate) struct QueryMethod {
    call: ExprCall,
}

impl Deref for QueryMethod {
    type Target = ExprCall;
    fn deref(&self) -> &Self::Target {
        &self.call
    }
}

impl DerefMut for QueryMethod {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.call
    }
}

impl Parse for QueryMethod {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            call: input.parse()?,
        })
    }
}

impl QueryMethod {
    pub fn name(&self) -> String {
        self.func.to_token_stream().to_string()
    }

    pub fn arg_count(&self) -> usize {
        self.args.len()
    }

    pub fn arg_usize(&self, index: usize) -> Option<usize> {
        let Some( arg ) = self.args.iter().nth( index ) else { return None };
        let Expr::Lit( ExprLit { lit: Lit::Int( i ), .. } ) = arg else { return None };

        match i.base10_parse::<usize>() {
            Ok(i) => Some(i),
            Err(_) => None,
        }
    }

    pub fn arg_str(&self, index: usize) -> Option<String> {
        let Some( arg ) = self.args.iter().nth( index ) else { return None };
        let Expr::Lit( ExprLit { lit: Lit::Str( s ), .. } ) = arg else { return None };
        Some(s.value())
    }

    pub fn apply_method_sql(&self, sql: &mut String) {
        let method_name = self.name();
        match method_name.as_str() {
            "shuffle" => quote_shuffle(self, sql),
            "pluck" => quote_pluck(self, sql),
            "limit" => quote_limit(self, sql),
            "count" => quote_count(self, sql),
            "page" => quote_page(self, sql),
            "one" => quote_one(self, sql),
            _ => {
                return emit_error!(
                    self.func, "Unrecognized shortcut method `{}`", method_name;
                    help = UNKNOWN_METHOD_HELP;
                    note = "See the crate documentation comments for a list of valid methods"
                )
            }
        }
    }
}

fn quote_shuffle(method: &QueryMethod, sql: &mut String) {
    *sql = match method.arg_count() {
        0 => format!("SELECT * FROM ({sql}) ORDER BY rand()"),

        1 if let Some( limit ) = method.arg_usize(0) && limit > 0 => {
            format!("SELECT * FROM ({sql}) ORDER BY rand() LIMIT {limit}")
        },

        _ => {
            return emit_error!(
                method.call, "Unrecognised Method arguments";
                help = r#"shuffle( [ <limit> ] )
- <limit>: usize - the maximum number of records to get"#,
            )
        }
    }
}

fn quote_pluck(method: &QueryMethod, sql: &mut String) {
    *sql = match method.arg_count() {
        1 if let Some( field_name ) = method.arg_str(0) => {
            format!("SELECT * FROM (SELECT {field_name} FROM ({sql}))")
        },

        2 if let Some( field_name ) = method.arg_str(0)
        && let Some( limit ) = method.arg_usize(1) => {
            format!("SELECT * FROM (SELECT {field_name} FROM ({sql}) LIMIT {limit})")
        },

        _ => {
            return emit_error!(
                method.call, "Unrecognised Method arguments";
                help = r#"pluck( <field> [ , <limit> ] ) 
- <field>: str - the name of the field to extract
- <limit>: usize - optional, the maximum number of records to get"#,
            )
        }
    }
}

fn quote_limit(method: &QueryMethod, sql: &mut String) {
    *sql = match method.arg_count() {
        1 if let Some( limit ) = method.arg_usize(0) && limit > 0 => {
            format!("SELECT * FROM ({sql}) LIMIT {limit}")
        },

        2 if let Some( limit ) = method.arg_usize(0) && limit > 0
        && let Some( start ) = method.arg_usize(1) => {
            format!("SELECT * FROM ({sql}) LIMIT {limit} START {start}")
        },

        _ => {
            return emit_error!(
                method.call, "Unrecognised Method arguments";
                help = r#"limit( <limit> [ , <start> ] )
- <limit>: usize - the maximum number of records to get
- <start>: usize - optional, "#,
            )
        }
    }
}

fn quote_count(method: &QueryMethod, sql: &mut String) {
    *sql = match method.arg_count() {
        0 => {
            format!("SELECT * FROM count(({sql}))")
        }

        _ => {
            return emit_error!(
                method.call, "Unrecognised Method arguments";
                help = r#"count( ) expects 0 args"#,
            )
        }
    }
}

fn quote_page(method: &QueryMethod, sql: &mut String) {
    *sql = match method.arg_count() {
        2 if let Some( size ) = method.arg_usize(0) && size > 0
        && let Some( page ) = method.arg_usize(1)  => {
            format!("SELECT * FROM ({sql}) LIMIT {} START {}", size, size*(page-1))
        },

        _ => {
            return emit_error!(
                method.call, "Unrecognised Method arguments";
                help = r#"page( <size> , <page> )
- <size>: usize - the number of records on each page
- <page>: usize - which page of records to get, 1 indexed"#,
            )
        }
    }
}

fn quote_one(method: &QueryMethod, sql: &mut String) {
    *sql = match method.arg_count() {
        0 => {
            format!("SELECT * FROM ({sql}) LIMIT 1")
        }

        _ => {
            return emit_error!(
                method.call, "Unrecognised Method arguments";
                help = r#"one( ) expects 0 args"#,
            )
        }
    }
}
