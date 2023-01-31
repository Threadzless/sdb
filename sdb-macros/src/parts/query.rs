// use proc_macro::TokenStream as TokenStreamOld;
use proc_macro2::{TokenStream, Span};
use quote::quote;
use syn::{parse::*, punctuated::Punctuated, token::*, *};

#[cfg(feature = "query-test")]
use crate::tester;
use crate::parts::*;

pub struct TransactionParse {
    pub async_tok: Option<Token!(!)>,
    pub client: Ident,
    pub args: Option<SdbArgs>,
    pub _arrow: FatArrow,
    pub _braces: Brace,
    pub lines: Punctuated<SdbStatement, Token![;]>,
}

impl Parse for TransactionParse {
    fn parse(input: ParseStream) -> Result<Self> {
        let context;
        let me = Self {
            async_tok: input.parse()?,
            client: input.parse()?,
            args: input.parse()?,
            _arrow: input.parse()?,
            _braces: braced!(context in input),
            lines: context.parse_terminated(SdbStatement::parse)?,
        };

        me.verify_formatting();

        Ok(me)
    }
}

impl TransactionParse {
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
                SdbStatement::Parse { .. } => parse_count += 1,
                SdbStatement::ParseTail { .. } => tail_count += 1,
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
            use SdbStatement as Ul;
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

        (push_steps, unpack, result_handle)
    }

    pub fn has_trailing(&self) -> bool {
        self.lines.iter().any(|l| match l {
            SdbStatement::ParseTail { .. } => true,
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
                SdbStatement::Import { var_name, .. } => {
                    vars.push((var_name.to_string(), line_num));
                }
                SdbStatement::ToVar { var_name, .. } => {
                    vars.push((var_name.to_string(), line_num));
                }
                _ => continue,
            }
        }
        vars
    }

    pub fn full_queries(&self) -> Vec<(&'_ LitStr, String)> {
        let mut queries = Vec::new();

        for line in self.lines.iter() {
            match line {
                SdbStatement::Import { .. } => continue,
                SdbStatement::Ignored { ref sql }
                | SdbStatement::ToVar { ref sql, .. }
                | SdbStatement::Parse { ref sql, .. }
                | SdbStatement::ParseTail { ref sql, .. } => {
                    queries.push(
                        (&sql.literal, sql.complete_sql())
                    )
                }
            }
        }

        queries
    }




    // pub fn syntax_check(&self) {
    //     tester::check_syntax(self)

    //     // let success = crate::syntaxer::check_syntax(&vars, &queries);

    //     #[cfg(feature = "query-test")]
    //     // if success.is_ok() {
    //         let full_sql = queries
    //             .iter()
    //             .map(|(sql, _)| sql.value())
    //             .collect::<Vec<String>>()
    //             .join(";\n");
    //     // }
    // }
}
