#![allow(unused)]
#![feature(proc_macro_quote, proc_macro_internals)]
#![feature(let_chains, async_closure)]
#![feature(if_let_guard)]

use proc_macro::TokenStream as TokenStreamOld;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse::*, punctuated::Punctuated, token::*, *};

use crate::*;

pub(crate) struct TransFunc {
    pub args: TransFuncArgs,
    pub flow: Option<Token![!]>,
    pub _split: FatArrow,
    pub lines: QueryLineList,
}

impl Parse for TransFunc {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            args: input.parse()?,
            flow: input.parse()?,
            _split: input.parse()?,
            lines: input.parse()?,
        })
    }
}

impl TransFunc {
    pub fn result_act(&self) -> TokenStream {
        match self.flow {
            Some(_) => quote!( .unwrap( ) ),
            None => quote!( ? ),
        }
    }

    pub fn arg_vars( &self) -> Vec<String> {
        let mut vars = Vec::new();
        for (idx, _field) in self.args.fields.iter().enumerate() {
            vars.push(format!("{idx}"))
        }
        for line in &self.lines.lines {
            match line {
                QueryLine::Let( l ) => {
                    vars.push( l.var.to_string() )
                },
                _ => continue
            }
        }
        vars
    }

    pub fn full_queries<'a>(&'a self) -> Vec<(String, &'a LitStr)> {
        let mut queries = Vec::new();
        for line in &self.lines.lines {
            match line {
                QueryLine::Raw( r ) => {
                    queries.push( (r.sql.value(), &r.sql) )
                },
                QueryLine::Let( l ) => {
                    match &l.input {
                        LetQueryInput::Query(q) => {
                            let full_sql = format!("LET ${} = ({})", l.var.to_string(), q.complete_sql());
                            queries.push( (full_sql, &q.sql) )        
                        },
                        _ => todo!()
                    }
                },
                // QueryLine::Let( LetQueryLine { input: LetQueryInput::} )
                QueryLine::Select( sel ) => {
                    queries.push( (sel.sql.complete_sql(), &sel.sql.sql) )
                },
                _ => continue
            }
        }
        queries
    }

    pub fn iter_lines(&self) -> impl Iterator<Item = &QueryLine> {
        self.lines.lines.iter()
    }

    pub fn extract_parts(
        &self,
        trans: &Ident,
        on_result: &TokenStream,
    ) -> (TokenStream, TokenStream) {
        let steps = (self.lines.lines)
            .iter()
            .map(|l| quote!( #l ))
            .collect::<TokenStream>();

        let outs = (self.lines.lines)
            .iter()
            .filter_map(|ql| ql.out_call(trans, &on_result))
            .collect::<TokenStream>();

        (steps, outs)
    }
}