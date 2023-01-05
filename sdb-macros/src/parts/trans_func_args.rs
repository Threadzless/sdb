use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse::*, punctuated::Punctuated, *};

// use super::*;

pub(crate) struct TransFuncArgs {
    pub _paren: token::Paren,
    pub client: Ident,
    pub _comma: Option<Token![,]>,
    pub fields: Punctuated<Expr, Token![,]>,
}

impl TransFuncArgs {
    pub fn field_assigns(&self) -> TokenStream {
        let mut assigns = TokenStream::new();
        for (idx, expr) in self.fields.iter().enumerate() {
            let span = match self._comma {
                Some(comma) => comma.span,
                None => Span::call_site(),
            };
            let var_name = LitStr::new(&idx.to_string(), span);
            assigns.extend(quote! {
                .push_var( #var_name, #expr )
            })
        }
        assigns
    }
}

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
