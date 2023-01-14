use syn::{parse::*, punctuated::Punctuated, *};

// use super::*;

pub struct TransFuncArgs {
    pub _paren: token::Paren,
    pub client: Ident,
    pub _comma: Option<Token![,]>,
    pub fields: Punctuated<Expr, Token![,]>,
}

impl TransFuncArgs {}

impl Parse for TransFuncArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let p_input;
        Ok(Self {
            _paren: parenthesized!(p_input in input),
            client: p_input.parse()?,
            _comma: p_input.parse()?,
            fields: p_input.parse_terminated(Expr::parse)?,
        })
    }
}
