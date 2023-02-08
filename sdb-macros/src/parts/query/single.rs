use ::proc_macro2::TokenStream;
use ::quote::quote;
use ::syn::{parse::*, token::*, *};

use crate::parts::{SdbArgs, TailStatement};


pub(crate) struct SingleQueryParse {
    pub async_tok: Option<Token!(!)>,
    pub client: Ident,
    pub args: Option<SdbArgs>,
    pub _arrow: FatArrow,
    pub stmt: TailStatement,
}

impl Parse for SingleQueryParse {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            async_tok: input.parse()?,
            client: input.parse()?,
            args: input.parse()?,
            _arrow: input.parse()?,
            stmt: input.parse()?,
        })
    }
}

impl SingleQueryParse {
    pub fn arg_steps(&self) -> TokenStream {
        match &self.args {
            Some(a) => a.field_assigns(),
            None => quote! {},
        }
    }

    pub fn prepare(&self, _trans_db: &Ident) -> (TokenStream, TokenStream, TokenStream) {
        let result_handle = match self.async_tok {
            Some(_) => quote! { .unwrap() },
            None => quote! { ? },
        };

        let TailStatement { sql, path, .. } = &self.stmt;
        let call = path.call_next();
        let unpack = quote! { #call };
        let push_steps = quote!( .push( #sql ) );

        (push_steps, unpack, result_handle)
    }

    pub fn arg_vars(&self) -> Vec<(String, usize)> {
        let mut vars = Vec::new();

        if let Some(args) = &self.args {
            for i in 0..args.fields.len() {
                vars.push((format!("{i}"), 0))
            }
            for name in args.arg_names() {
                vars.push((name, 0))
            }
        }

        vars
    }

    pub fn full_queries(&self) -> Vec<(&'_ LitStr, String)> {
        let TailStatement { ref sql, .. } = self.stmt;
        vec![
            ( &sql.literal, sql.complete_sql() )
        ]
    }
}