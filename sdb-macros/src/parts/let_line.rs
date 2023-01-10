use proc_macro2::TokenStream;
use proc_macro_error::abort;
// use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::*, token::*, *};

use crate::QuerySqlBlock;

pub(crate) struct LetQueryLine {
    pub _dollar: Dollar,
    pub var: Ident,
    pub _eq: syn::token::Eq,
    pub input: LetQueryInput,
}

impl Parse for LetQueryLine {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            _dollar: input.parse()?,
            var: input.parse()?,
            _eq: input.parse()?,
            input: input.parse()?,
        })
    }
}

impl ToTokens for LetQueryLine {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let var = LitStr::new(self.var.to_string().as_str(), self.var.span());

        use LetQueryInput::*;
        tokens.extend(match &self.input {
            Query(query) => {
                let base_sql = &query.sql;
                let sql_lit = LitStr::new(&base_sql.value(), base_sql.span());
                quote!(
                    .query_to_var( #var, #sql_lit )
                )
            }
            Block(expr) => quote!(
                .push_var( #var, #expr )
            ),
            Paren(expr) => quote!(
                .push_var( #var, #expr )
            ),
        })
    }
}

pub(crate) enum LetQueryInput {
    Query(QuerySqlBlock),
    Block(ExprBlock),
    Paren(ExprParen),
}

impl Parse for LetQueryInput {
    fn parse(input: ParseStream) -> Result<Self> {
        if let Ok(sql) = input.parse::<QuerySqlBlock>() {
            sql.check(None);
            Ok(Self::Query(sql))
        } else if let Ok(expr) = input.parse::<ExprParen>() {
            Ok(Self::Paren(expr))
        } else if let Ok(expr) = input.parse::<ExprBlock>() {
            Ok(Self::Block(expr))
        } else {
            abort!(
                input.span(),
                "Expected a SQL string or brackets with an expression"
            );
        }
    }
}
