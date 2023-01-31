use std::fmt::{Debug, Formatter, Result as FmtResult};

use proc_macro2::{TokenStream, Span};
use proc_macro_error::{emit_error, emit_warning};
use quote::{quote, ToTokens};
use syn::{parse::*, LitStr, Token, token::Paren, parenthesized};

use super::QueryMethod;

// use crate::sql::queries::*;

pub struct QuerySqlBlock {
    pub literal: LitStr,
    pub methods: Vec<QueryMethod>,
}

impl Parse for QuerySqlBlock {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut me = Self {
            literal: input.parse()?,
            methods: Vec::new(),
        };

        while input.peek(Token![.]) {
            me.methods.push(input.parse::<QueryMethod>()?)
        }

        me.check();

        Ok(me)
    }
}

impl ToTokens for QuerySqlBlock {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let sql = self.complete_sql();
        let sql_lit = LitStr::new(&sql, self.literal.span());
        tokens.extend(quote!( #sql_lit ));
    }
}

impl Debug for QuerySqlBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("QuerySqlBlock")
            .field("methods", &self.methods)
            .field("sql", &self.literal)
            .finish()
    }
}

impl QuerySqlBlock {
    /// Emits an error if the SQL string contains any semicolons, and
    /// a emits a warning if the query contains comments
    pub fn check(&self) {
        let sql_span = &self.literal.span();
        let sql_str = self.literal.value();

        if sql_str.contains("--") {
            emit_warning!( sql_span, "Queries shouldn't contain comments";
                help = "Remove doube minuses (--) from you query.\nIf you need to put comments here, put them in the macro using //"
            )
        }

        let Some( (first, second) ) = sql_str.split_once(';') else { return };

        let suggestion = format!("{first:?};\n{second:?}");

        emit_error!( sql_span, "Queries cannot contain semicolons (;)";
            help = "Try splitting the query into seperate query strings:\n\n{}", suggestion;
        )
    }

    pub(crate) fn build_methods(&self, mut sql: String) -> Option<String> {
        if self.methods.len() == 0 {
            return None
        }

        for method in &self.methods {
            method.apply_method_sql(&mut sql)
        }
        
        Some( sql )
    }

    pub fn complete_sql(&self) -> String {
        let mut sql = self.literal.value();

        match self.build_methods(sql) {
            Some(full_sql) => full_sql,
            None => self.literal.value(),
        }
    }
}