// use proc_macro::TokenStream as TokenStreamOld;
use proc_macro2::{TokenStream, Span};
use quote::{quote, ToTokens};
use syn::{parse::*, punctuated::Punctuated, token::*, *};

#[cfg(feature = "query-test")]
use crate::tester;
use crate::parts::*;

pub struct InsertParse {
    pub client: Ident,
    pub _arrow1: FatArrow,
    pub ignore: Option<Token![!]>,
    pub table: Ident,
    pub fields: ExprTuple,
    pub _arrow2: FatArrow,
    pub values: InsertValues,
}

impl Parse for InsertParse {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            client: input.parse()?,
            _arrow1: input.parse()?,
            ignore: input.parse()?,
            table: input.parse()?,
            fields: input.parse()?,
            _arrow2: input.parse()?,
            values: input.parse()?,
        })
    }
}

//

//

//

pub struct InsertValues {
    pub brack: Bracket,
    pub rows: Punctuated<ExprTuple, Token![,]>,
}

impl Parse for InsertValues {
    fn parse(input: ParseStream) -> Result<Self> {
        let ctx;
        Ok(Self {
            brack: bracketed!(ctx in input),
            rows: ctx.parse_terminated(ExprTuple::parse)?,
        })
    }
}

impl ToTokens for InsertValues {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.brack.surround(tokens, |t| self.rows.to_tokens(t) )
    }
}