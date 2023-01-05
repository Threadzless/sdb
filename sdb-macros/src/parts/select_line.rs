use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::*, token::*, *};

use super::*;

#[derive(Debug)]
pub(crate) struct SelectQueryLine {
    pub r#mut: Option<Mut>,
    pub into: Ident,
    pub cast: QueryResultType,
    pub _eq: Option<Eq>,
    pub sql: QuerySqlBlock,
}

impl SelectQueryLine {
    pub fn method_call(&self) -> TokenStream {
        use QueryResultScale::*;

        match &self.cast.scale {
            Option(_to) => quote!(next_option()),
            Single(_to) => quote!(next_one()),
            Vec(_to) => quote!(next_vec()),
        }
    }

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
            r#mut: input.parse()?,
            into: input.parse()?,
            cast: input.parse()?,
            _eq: input.parse()?,
            sql: input.parse()?,
        };

        me.sql.check(Some((&me.into, &me.cast, me.mut_token())));

        Ok(me)
    }
}

impl ToTokens for SelectQueryLine {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let full_sql = &self.sql;
        tokens.extend(quote! {
            .push( false, #full_sql )
        })
    }
}
