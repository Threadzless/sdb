use proc_macro2::{Delimiter, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{buffer::Cursor, parse::*, punctuated::Punctuated, token::CustomToken, *};

// use super::*;

pub struct QueryArgs {
    pub _eq: Token![=],
    pub _bracket: token::Bracket,
    pub fields: Punctuated<Expr, Token![,]>,
}

impl QueryArgs {
    pub fn field_assigns(&self) -> TokenStream {
        let mut assigns = TokenStream::new();
        for (idx, expr) in self.fields.iter().enumerate() {
            let span = Span::call_site();
            let var_name = LitStr::new(&idx.to_string(), span);
            assigns.extend(quote! {
                .push_var( #var_name, #expr )
            })
        }
        assigns
    }

    pub fn arg_names(&self) -> Vec<String> {
        let mut names = vec![];
        for field in &self.fields {
            match field {
                Expr::Cast(cast) => {
                    let name = cast.ty.to_token_stream().to_string();
                    names.push(name)
                }
                Expr::Path(path) if path.path.segments.len() == 1 => {
                    let name = path.path.segments.first().unwrap().ident.to_string();
                    names.push(name)
                }
                _ => continue,
            }
        }

        names
    }
}

impl Parse for QueryArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let p_input;
        Ok(Self {
            _eq: input.parse()?,
            _bracket: bracketed!(p_input in input),
            fields: p_input.parse_terminated(Expr::parse)?,
        })
    }
}

impl ToTokens for QueryArgs {
    fn to_tokens(&self, _tokens: &mut TokenStream) {
        todo!()
    }
}

impl CustomToken for QueryArgs {
    fn peek(cursor: Cursor) -> bool {
        let Some((p0, cur)) = cursor.punct() else { return false };
        if p0.as_char().ne(&'=') {
            return false;
        };
        let Some((_inside, _, after)) = cur.group(Delimiter::Bracket) else { return false };
        let Some((p1, cur) ) = after.punct() else { return false };
        if p1.as_char() != '=' {
            return false;
        };
        let Some((p2, _)) = cur.punct() else { return false };
        p2.as_char() == '>'
    }

    fn display() -> &'static str {
        todo!()
    }
}
