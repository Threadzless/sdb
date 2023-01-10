#![allow(unused)]
#![feature(proc_macro_quote, proc_macro_internals)]
#![feature(let_chains, async_closure)]
#![feature(if_let_guard)]

use proc_macro::TokenStream as TokenStreamOld;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse::*, punctuated::Punctuated, token::*, *};

use crate::*;

pub(crate) struct QueryFunc {
    pub async_tok: Option<Token!(!)>,
    pub args: TransFuncArgs,
    pub _split: FatArrow,
    pub line: SelectQueryLineAnon,
}

impl Parse for QueryFunc {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            async_tok: input.parse()?,
            args: input.parse()?,
            _split: input.parse()?,
            line: input.parse()?,
        })
    }
}

impl QueryFunc {
    pub fn extract_parts(
        &self,
        trans: &Ident,
        on_result: &TokenStream,
    ) -> (TokenStream, Option<TokenStream>) {
        let line = &self.line;
        let steps = quote! { #line };
        let outs = self.line.method_call(); //trans, &on_result);

        (steps, Some(outs))
    }
}
