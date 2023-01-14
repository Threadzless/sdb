// use proc_macro::TokenStream as TokenStreamOld;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::*, punctuated::Punctuated, token::*, *};

use crate::parts::*;

pub struct QueryFunc {
    pub async_tok: Option<Token!(!)>,
    pub client: Ident,
    pub args: Option<QueryArgs>,
    pub _arrow: FatArrow,
    pub _braces: Brace,
    pub lines: Punctuated<UniversalLine, Token![;]>,
}

impl Parse for QueryFunc {
    fn parse(input: ParseStream) -> Result<Self> {
        let context;
        let me = Self {
            async_tok: input.parse()?,
            client: input.parse()?,
            args: input.parse()?,
            _arrow: input.parse()?,
            _braces: braced!(context in input),
            lines: context.parse_terminated(UniversalLine::parse)?,
        };

        me.verify_formatting();

        Ok(me)
    }
}

impl QueryFunc {
    pub fn arg_steps(&self) -> TokenStream {
        match &self.args {
            Some(a) => a.field_assigns(),
            None => quote! {},
        }
    }

    pub fn verify_formatting(&self) {
        let mut tail_count = 0;
        let mut parse_count = 0;
        for line in &self.lines {
            match line {
                UniversalLine::Parse { .. } => parse_count += 1,
                UniversalLine::ParseTail { .. } => tail_count += 1,
                _ => continue,
            }
        }

        if tail_count > 0 && parse_count > 0 {
            // emit_error!(

            // )
        }
    }

    pub fn prepare(&self, trans_db: &Ident) -> (TokenStream, TokenStream, TokenStream) {
        let result_handle = match self.async_tok {
            Some(_) => quote! { .unwrap() },
            None => quote! { ? },
        };

        let mut unpack = TokenStream::new();
        let mut push_steps = TokenStream::new();

        for line in &self.lines {
            use UniversalLine as Ul;
            match line {
                Ul::Import {
                    source,
                    _dollar,
                    var_name,
                    ..
                } => {
                    let span = var_name
                        .span()
                        .join(_dollar.span)
                        .unwrap_or(var_name.span());
                    let var_str = LitStr::new(var_name.to_string().as_str(), span);
                    push_steps.extend(quote!( .push_var( #var_str, ( #source ) ) ))
                }

                Ul::ToVar {
                    sql,
                    _dollar,
                    var_name,
                    ..
                } => {
                    let span = var_name
                        .span()
                        .join(_dollar.span)
                        .unwrap_or(var_name.span());
                    let var_str = LitStr::new(var_name.to_string().as_str(), span);
                    push_steps.extend(quote!( .query_to_var( #var_str, #sql ) ))
                }

                Ul::Ignored { sql } => push_steps.extend(quote!( .push_skipped( #sql ) )),

                Ul::Parse {
                    sql,
                    is_mut,
                    store,
                    path,
                    ..
                } => {
                    push_steps.extend(quote!( .push( #sql ) ));
                    let call = path.call_next();
                    unpack.extend(quote! {
                        let #is_mut #store = #trans_db . #call #result_handle;
                    })
                }

                Ul::ParseTail { sql, _as, path } => {
                    push_steps.extend(quote!( .push( #sql ) ));
                    let call = path.call_next();

                    unpack = quote! { #call #result_handle };
                }
            }
        }

        return (push_steps, unpack, result_handle);
    }

    pub fn has_trailing(&self) -> bool {
        self.lines.iter().any(|l| match l {
            UniversalLine::ParseTail { .. } => true,
            _ => false,
        })
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

        for (line_num, line) in self.lines.iter().enumerate() {
            match line {
                UniversalLine::Import { var_name, .. } => {
                    vars.push((var_name.to_string(), line_num));
                }
                UniversalLine::ToVar { var_name, .. } => {
                    vars.push((var_name.to_string(), line_num));
                }
                _ => continue,
            }
        }
        vars
    }

    pub fn full_queries(&self) -> Vec<(String, &LitStr)> {
        let mut queries = Vec::new();

        for line in self.lines.iter() {
            match line {
                UniversalLine::Import { .. } => continue,
                UniversalLine::Ignored { sql }
                | UniversalLine::ToVar { sql, .. }
                | UniversalLine::Parse { sql, .. }
                | UniversalLine::ParseTail { sql, .. } => {
                    queries.push((sql.complete_sql(), &sql.literal))
                }
            }
        }

        queries
    }
}
