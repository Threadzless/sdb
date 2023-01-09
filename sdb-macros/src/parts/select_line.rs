use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::*, token::*, *};

use super::*;

#[derive(Debug)]
pub(crate) struct SelectQueryLine {
    pub r#mut: Option<Mut>,
    pub into: Ident,
    pub _colon: Colon,
    pub cast: QueryResultType,
    pub _eq: Option<Eq>,
    pub sql: QuerySqlBlock,
}

impl SelectQueryLine {
    pub fn method_call(&self) -> TokenStream {
        use QueryResultScale::*;

        match &self.cast.scale {
            Option(_to) => quote!(next_one()),
            Single(_to) => quote!(next_one_exact()),
            Vec(_to) => quote!(next_list()),
        }
    }

    pub fn mut_token(&self) -> TokenStream {
        // todo!()
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
            _colon: input.parse()?,
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

//

//

//

//

//


#[derive(Debug)]
pub(crate) struct SelectQueryLineAnon {
    pub cast: QueryResultType,
    pub _eq: Option<Eq>,
    pub sql: QuerySqlBlock,
}

impl SelectQueryLineAnon {
    pub fn method_call(&self) -> TokenStream {
        use QueryResultScale::*;

        match &self.cast.scale {
            Option(_to) => quote!(next_one()),
            Single(_to) => quote!(next_one_exact()),
            Vec(_to) => quote!(next_list()),
        }
    }
}

impl Parse for SelectQueryLineAnon {
    fn parse(input: ParseStream) -> Result<Self> {
        let me = Self {
            cast: input.parse()?,
            _eq: input.parse()?,
            sql: input.parse()?,
        };

        me.sql.check(None);//Some((&me.into, &me.cast, me.mut_token())));

        Ok(me)
    }
}

impl ToTokens for SelectQueryLineAnon {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let full_sql = &self.sql;
        tokens.extend(quote! {
            .push( false, #full_sql )
        })
    }
}
