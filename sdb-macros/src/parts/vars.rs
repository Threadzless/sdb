use syn::{parse::*, punctuated::Punctuated, *};

// use super::*;

pub(crate) struct TransactionVars {
    pub _paren: token::Paren,
    pub _client: Ident,
    pub _comma: Option<Token![,]>,
    pub _fields: Punctuated<Expr, Token![,]>,
}

impl TransactionVars {}

impl Parse for TransactionVars {
    fn parse(input: ParseStream) -> Result<Self> {
        let p_input;
        Ok(Self {
            _paren: parenthesized!(p_input in input),
            _client: p_input.parse()?,
            _comma: p_input.parse()?,
            _fields: p_input.parse_terminated(Expr::parse)?,
        })
    }
}
