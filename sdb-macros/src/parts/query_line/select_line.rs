use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::*, token::*, *};

use crate::parts::*;

#[derive(Debug)]
pub struct SelectQueryLine {
    pub sql: QuerySqlBlock,
    pub _arrow: Token![->],
    pub r#mut: Option<Mut>,
    pub into: Ident,
    pub _colon: Colon,
    pub cast: QueryResultType,
    
}

impl SelectQueryLine {
    pub fn mut_token(&self) -> TokenStream {
        match &self.r#mut {
            Some(m) => quote! { #m },
            None => quote! {},
        }
    }
}

impl Parse for SelectQueryLine {
    fn parse(input: ParseStream) -> Result<Self> {
        let me = Self {
            sql: input.parse()?,
            _arrow: input.parse()?,
            r#mut: input.parse()?,
            into: input.parse()?,
            _colon: input.parse()?,
            cast: input.parse()?,
        };

        me.sql.check(Some((&me.into, &me.cast, me.mut_token())));

        Ok(me)
    }
}

impl ToTokens for SelectQueryLine {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let full_sql = &self.sql;
        tokens.extend(quote! {
            .push( #full_sql )
        })
    }
}
