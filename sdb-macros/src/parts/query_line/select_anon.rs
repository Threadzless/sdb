use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::*, *};

use crate::parts::*;

#[derive(Debug)]
pub struct SelectQueryLineAnon {
    pub sql: QuerySqlBlock,
    pub _arrow: Token!(->),
    pub _cast: QueryResultType,
}

// impl SelectQueryLineAnon {
//     pub fn method_call(&self) -> TokenStream {
//         use QueryResultScale::*;

//         match &self._cast.scale {
//             Option(_to) => quote!(next_one()),
//             Single(_to) => quote!(next_one_exact()),
//             Vec(_to) => quote!(next_list()),
//         }
//     }
// }

impl Parse for SelectQueryLineAnon {
    fn parse(input: ParseStream) -> Result<Self> {
        let me = Self {
            sql: input.parse()?,
            _arrow: input.parse()?,
            _cast: input.parse()?,
        };

        me.sql.check(None); //Some((&me.into, &me.cast, me.mut_token())));

        Ok(me)
    }
}

impl ToTokens for SelectQueryLineAnon {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let full_sql = &self.sql;
        tokens.extend(quote! {
            .push( #full_sql )
        })
    }
}
