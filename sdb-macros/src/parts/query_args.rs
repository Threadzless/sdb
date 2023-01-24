use proc_macro2::{Delimiter, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{buffer::Cursor, parse::*, punctuated::Punctuated, token::CustomToken, *};

// use super::*;

pub struct QueryArgs {
    pub _eq: Token![=],
    pub _bracket: token::Bracket,
    pub fields: Punctuated<QueryArg, Token![,]>,
}

impl QueryArgs {
    pub fn field_assigns(&self) -> TokenStream {
        let mut assigns = TokenStream::new();
        for (idx, arg) in self.fields.iter().enumerate() {
            let var_num = LitStr::new(&idx.to_string(), Span::call_site());

            match arg {
                QueryArg::Expr( expr ) => {
                    assigns.extend(quote! {
                        .push_var( #var_num, #expr )
                    });
                },
                QueryArg::Var( ident ) => {
                    let var_name = LitStr::new(&ident.to_string(), ident.span());
                    assigns.extend(quote! {
                        .push_var( #var_num, #ident )
                        ._name_var( #var_name, #var_num )
                    });
                },
                QueryArg::Alias { name, expr, .. } => {
                    let var_name = LitStr::new(&name.to_string(), name.span());
                    assigns.extend(quote! {
                        .push_var( #var_num, #expr )
                        ._name_var( #var_name, #var_num )
                    });
                },
            }
        }
        assigns
    }

    pub fn arg_names(&self) -> Vec<String> {
        let mut names = vec![];
        for (index, field) in self.fields.iter().enumerate() {
            names.push(format!("{index}"));
            match field {
                QueryArg::Alias { name, .. } => names.push(name.to_string()),
                QueryArg::Var( ident ) => names.push(ident.to_string()),
                _ => { } 
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
            fields: p_input.parse_terminated(QueryArg::parse)?,
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



pub enum QueryArg {

    Expr( Expr ),
    Var( Ident ),
    Alias {
        name: Ident,
        _colon: Token![:],
        expr: Expr,
    }
}


impl Parse for QueryArg {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek2(Token![:]) {
            return Ok(Self::Alias {
                name: input.parse()?,
                _colon: input.parse()?,
                expr: input.parse()?,
            })
        }
        match input.parse::<Ident>() {
            Err(_) => Ok(Self::Expr( input.parse()? )),
            Ok( ident ) => Ok(Self::Var( ident )),
        }
    }
}