use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::*, *};

pub struct RawQueryLine {
    pub sql: LitStr,
}

impl Parse for RawQueryLine {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            sql: input.parse()?,
        })
    }
}

impl ToTokens for RawQueryLine {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let sql = &self.sql;
        tokens.to_tokens(&mut quote!( .push_skipped( #sql ) ))
    }
}
