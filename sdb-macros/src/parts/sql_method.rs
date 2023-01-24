use std::fmt::Debug;

use proc_macro2::Delimiter;
use proc_macro_error::emit_error;
use quote::ToTokens;
use syn::{parse::*, punctuated::Punctuated, token::CustomToken, *};

const UNKNOWN_METHOD_HELP: &str = r#"Valid Query Sugar™s:
  - count     - ids       - limit
  - one       - page      - pluck
  - product   - shuffle   - sum
"#;

#[derive(Debug)]
pub struct QueryMethod {
    _dot: Token![.],
    ident: Ident,
    _paren: token::Paren,
    args: Punctuated<QuerySugarArg, Token![,]>,
}

impl Parse for QueryMethod {
    fn parse(input: ParseStream) -> Result<Self> {
        let context;
        Ok(Self {
            _dot: input.parse()?,
            ident: input.parse()?,
            _paren: parenthesized!(context in input),
            args: context.parse_terminated(QuerySugarArg::parse)?,
        })
    }
}

impl CustomToken for QueryMethod {
    fn peek(cursor: buffer::Cursor) -> bool {
        let cur = match cursor.punct() {
            Some((punct, cur)) if punct.as_char().eq(&'.') => cur,
            _ => cursor,
        };
        let Some((_, cur)) = cur.ident() else { return false };
        let Some((_, _, _)) = cur.group(Delimiter::Parenthesis) else { return false };

        true
    }

    fn display() -> &'static str {
        todo!()
    }
}

impl QueryMethod {
    pub fn name(&self) -> String {
        self.ident.to_token_stream().to_string()
    }

    pub fn arg_count(&self) -> usize {
        self.args.len()
    }

    pub fn arg_usize(&self, index: usize) -> Option<usize> {
        let Some( arg ) = self.args.iter().nth( index ) else { return None };
        match arg {
            &QuerySugarArg::Literal( Lit::Int( ref i ) ) => i.base10_parse::<usize>().ok(),
            _ => None
        }
    }

    pub fn arg_str(&self, index: usize) -> Option<String> {
        let Some( arg ) = self.args.iter().nth( index ) else { return None };
        match arg {
            &QuerySugarArg::Literal( Lit::Str( ref s ) ) => Some( s.value() ),
            _ => None
        }
    }

    #[allow(unused)]
    pub fn arg_ident(&self, index: usize) -> Option<&Ident> {
        let Some( arg ) = self.args.iter().nth( index ) else { return None };
        match arg {
            &QuerySugarArg::Ident( ref ident ) => Some( ident ),
            _ => None,
        }
    }

    pub fn apply_method_sql(&self, sql: &mut String) {
        let method_name = self.name();
        match method_name.as_str() {
            "count" => quote_count(self, sql),
            "ids" => quote_ids(self, sql),
            "limit" => quote_limit(self, sql),
            "one" => quote_one(self, sql),
            "page" => quote_page(self, sql),
            "pluck" => quote_pluck(self, sql),
            "product" => quote_product(self, sql),
            "shuffle" => quote_shuffle(self, sql),
            "sum" => quote_sum(self, sql),
            _ => {
                emit_error!(
                    self.ident, "Unrecognized Query Sugar™ `{}`", method_name;
                    help = UNKNOWN_METHOD_HELP;
                    note = "See the crate documentation comments for a list of Query Sugar™"
                )
            }
        }
    }
}

fn quote_ids(method: &QueryMethod, sql: &mut String) {
    *sql = match method.arg_count() {
        0 => format!("SELECT * FROM (SELECT type::string( `id` ) AS id FROM ({sql}))"),

        _ => {
            return emit_error!(
                method.ident, "Invalid Query Sugar™ Arguments";
                help = r#"ids() expects no arguments"#,
            )
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
                method.ident, "Invalid Query Sugar™ Arguments";
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
                method.ident, "Invalid Query Sugar™ Arguments";
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
                method.ident, "Invalid Query Sugar™ Arguments";
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
        },

        1 if let Some( field ) = method.arg_str(0) => {
            format!("SELECT * FROM count((SELECT * FROM (SELECT {field} FROM ({sql}))))")
        },

        _ => {
            return emit_error!(
                method.ident, "Invalid Query Sugar™ Arguments";
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
                method.ident, "Invalid Query Sugar™ Arguments";
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
                method.ident, "Invalid Query Sugar™ Arguments";
                help = r#"one( ) expects 0 args"#,
            )
        }
    }
}

fn quote_product(method: &QueryMethod, sql: &mut String) {
    *sql = match method.arg_count() {
        1 if let Some( field ) = method.arg_str(0) => {
            let inner = format!("SELECT * FROM (SELECT `{field}` FROM ({sql}))");
            format!("SELECT * FROM math::product(({inner}))")
        },
        _ => {
            return emit_error!(
                method.ident, "Invalid Query Sugar™ Arguments";
                help = r#"product( <field> ) expects 1 args
- <field>: string - the field to get the product of"#,
            )
        }
    }
}

fn quote_sum(method: &QueryMethod, sql: &mut String) {
    *sql = match method.arg_count() {
        1 if let Some( field ) = method.arg_str(0) => {
            let inner = format!("SELECT * FROM (SELECT `{field}` FROM ({sql}))");
            format!("SELECT * FROM math::sum(({inner}))")
        },
        _ => {
            return emit_error!(
                method.ident, "Invalid Query Sugar™ Arguments";
                help = r#"sum( <field> ) expects 1 args
- <field>: string - the field to get the sum of"#,
            )
        }
    }
}


#[derive(Debug)]
pub enum QuerySugarArg {
    Literal(Lit),
    TransVar(Token![$], Ident),
    Ident( Ident ),
}

impl Parse for QuerySugarArg {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![$]) {
            Ok(Self::TransVar(
                input.parse()?,
                input.parse()?,
            ))
        }
        else if input.peek(Lit) {
            Ok(Self::Literal( input.parse()? ))
        }
        else {
            Ok(Self::Ident( input.parse()? ))
        }
    }
}