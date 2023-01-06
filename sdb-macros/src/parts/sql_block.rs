use std::fmt::Debug;

use proc_macro2::TokenStream;
use proc_macro_error::{emit_error, emit_warning};
use quote::{quote, ToTokens};
use syn::{parse::*, token::*, *};

use super::QueryResultType;

const UNKNOWN_METHOD_HELP: &str = r#"Expected one of the following methods, or no method:
 - pluck
 - limit
 - shuffle
 - count
 - one
 - page
"#;

pub(crate) struct QuerySqlBlock {
    pub methods: Vec<QueryMethod>,
    pub sql: LitStr,
}

impl Parse for QuerySqlBlock {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut methods = Vec::new();
        while !input.peek(LitStr) && input.peek2(Paren) {
            methods.push(input.parse()?)
        }

        Ok(Self {
            methods,
            sql: input.parse()?,
        })
    }
}

impl ToTokens for QuerySqlBlock {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut sql = self.sql.value();

        for method in &self.methods {
            self.quote_method(method, &mut sql)
        }

        let sql_lit = LitStr::new(&sql, self.sql.span());

        tokens.extend(quote!( #sql_lit ));
    }
}

impl QuerySqlBlock {
    fn quote_method(&self, method: &QueryMethod, sql: &mut String) {
        let method_name = method.name();
        match method_name.as_str() {
            "shuffle" => self.quote_shuffle(method, sql),
            "pluck" => self.quote_pluck(method, sql),
            "limit" => self.quote_limit(method, sql),
            "count" => self.quote_count(method, sql),
            "page" => self.quote_page(method, sql),
            "one" => self.quote_one(method, sql),
            _ => {
                emit_error!(
                    method.call, "Unrecognized shortcut method `{}`", method_name;
                    help = UNKNOWN_METHOD_HELP;
                    note = "See the crate documentation comments for a list of valid methods"
                )
            }
        }
    }

    fn quote_shuffle(&self, method: &QueryMethod, sql: &mut String) {
        *sql = match method.args.len() {
            0 => format!("SELECT * FROM ({sql}) ORDER BY rand()"),

            1 if let Some( limit ) = method.arg_usize(0) && limit > 0 => {
                format!("SELECT * FROM ({sql}) ORDER BY rand() LIMIT {limit}")
            },

            _ => {
                return emit_error!(
                    method.call, "Unrecognised Method args.";
                    help = r#"shuffle() expects 0 args"#,
                )
            }
        }
    }

    fn quote_pluck(&self, method: &QueryMethod, sql: &mut String) {
        *sql = match method.args.len() {
            1 if let Some( field_name ) = method.arg_str(0) => {
                format!("SELECT * FROM (SELECT {field_name} FROM ({sql}))")
            },

            _ => {
                return emit_error!(
                    method.call, "Unrecognised Method args.";
                    help = r#"pluck() expects 1 arg,  the name of a field to extract from the result(s)"#,
                )
            }
        }
    }

    fn quote_limit(&self, method: &QueryMethod, sql: &mut String) {
        *sql = match method.args.len() {
            1 if let Some( limit ) = method.arg_usize(0)
            && limit > 0 => {
                format!("SELECT * FROM ({sql}) LIMIT {limit}")
            },

            2 if let Some( limit ) = method.arg_usize(0) && limit > 0 
            && let Some( start ) = method.arg_usize(1) => {
                format!("SELECT * FROM ({sql}) LIMIT {limit} START {start}")
            },

            _ => {
                return emit_error!(
                    method.call, "Inrecognised Method args.";
                    help = r#"limit() expects one arg, the maximum number of records to retrieve"#,
                )
            }
        }
    }

    fn quote_count(&self, method: &QueryMethod, sql: &mut String) {
        match method.args.len() {
            0 => {
                *sql = format!("SELECT * FROM count(({sql}))");
            }
            _ => {
                emit_error!(
                    method.call, "Unrecognised Method args.";
                    help = r#"count() expects 0 args"#,
                )
            }
        }
    }

    fn quote_page(&self, method: &QueryMethod, sql: &mut String) {
        *sql = match method.args.len() {
            2 if let Some( size ) = method.arg_usize(0) && size > 0
            && let Some( page ) = method.arg_usize(1)  => {
                format!("SELECT * FROM ({sql}) LIMIT {} START {}", size, size*(page-1))
            },

            _ => {
                return emit_error!(
                    method.call, "Unrecognised Method args.";
                    help = r#"page() expects 2 args, a page size, and the page number"#,
                )
            }
        }
    }

    fn quote_one(&self, method: &QueryMethod, sql: &mut String) {
        match method.args.len() {
            0 => {
                *sql = format!("SELECT * FROM ({sql}) LIMIT 1");
            }

            _ => {
                emit_error!(
                    method.call, "Unrecognised Method args.";
                    help = r#"one() expects 0 args"#,
                )
            }
        }
    }

    /// Emits an error if the SQL string contains any semicolons, and
    /// a emits a warning if the query contains comments
    pub fn check(&self, context: Option<(&Ident, &QueryResultType, TokenStream)>) {
        let sql_token = &self.sql;
        let sql_str = sql_token.value();

        if sql_str.contains("--") {
            emit_warning!( sql_token, "Queries shouldn't contain comments";
                help = "Remove doube minuses (--) from you query.\nIf you need to put comments here, put them in the macro using //"
            )
        }

        let Some( (first, second) ) = sql_str.split_once(";") else { return };

        if let Some((ident, cast, mut_token)) = context {
            let var_ident = ident.to_string();
            let var_type = cast.cast_type().to_string();

            let suggestion =
                format!("{first:?};\n{mut_token} {var_ident}: {var_type} = {second:?}");

            emit_error!( sql_token, "Queries cannot contain semicolons (;)";
                help = "Try splitting the query into seperate query strings:\n\n{}", suggestion;
            )
        } else {
            emit_error!( sql_token, "Queries cannot contain semicolons (;)";
                help = "Try splitting the query into seperate query strings";
            )
        }
    }
}

impl Debug for QuerySqlBlock {
    fn fmt(&self, f: &mut __private::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuerySqlBlock")
            .field("methods", &self.methods)
            .field("sql", &self.sql)
            .finish()
    }
}

//

//

//

//

#[derive(Debug)]
pub(crate) struct QueryMethod {
    pub call: ExprCall,
    // pub name: Ident,
    // pub args: Vec<Lit>,
}

impl QueryMethod {
    pub fn name(&self) -> String {
        self.call.func.to_token_stream().to_string()
    }

    // pub fn arg_isize( &self, index: usize ) -> Option<isize> {
    //     let Some( arg ) = self.call.args.iter().nth( index ) else { return None };
    //     let Expr::Lit( ExprLit { lit: Lit::Int( i ), .. } ) = arg else { return None };

    //     match i.base10_parse::<isize>() {
    //         Ok( i ) => Some( i ),
    //         Err( _ ) => None,
    //     }
    // }

    pub fn arg_usize(&self, index: usize) -> Option<usize> {
        let Some( arg ) = self.call.args.iter().nth( index ) else { return None };
        let Expr::Lit( ExprLit { lit: Lit::Int( i ), .. } ) = arg else { return None };

        match i.base10_parse::<usize>() {
            Ok(i) => Some(i),
            Err(_) => None,
        }
    }

    pub fn arg_str(&self, index: usize) -> Option<String> {
        let Some( arg ) = self.call.args.iter().nth( index ) else { return None };
        let Expr::Lit( ExprLit { lit: Lit::Str( s ), .. } ) = arg else { return None };
        Some(s.value())
    }
}

impl Parse for QueryMethod {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            call: input.parse()?,
        })
    }
}

impl std::ops::Deref for QueryMethod {
    type Target = ExprCall;
    fn deref(&self) -> &Self::Target {
        &self.call
    }
}
