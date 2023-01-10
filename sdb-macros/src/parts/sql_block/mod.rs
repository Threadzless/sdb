use std::fmt::Debug;

use proc_macro2::TokenStream;
use proc_macro_error::{emit_error, emit_warning};
use quote::{quote, ToTokens};
use syn::{parse::*, token::*, *};

use super::QueryResultType;

mod sql_method;
pub(crate) use sql_method::*;

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
        let sql = self.complete_sql();

        let sql_lit = LitStr::new(&sql, self.sql.span());

        tokens.extend(quote!( #sql_lit ));
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

impl QuerySqlBlock {
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

    pub fn complete_sql(&self) -> String {
        let mut sql = self.sql.value();

        for method in &self.methods {
            method.apply_method_sql(&mut sql)
        }

        sql
    }
}
