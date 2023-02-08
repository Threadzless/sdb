use ::proc_macro2::TokenStream;
use ::quote::quote;
use ::syn::{parse::*, punctuated::Punctuated, token::*, *};

use crate::parts::{SdbArgs, SdbStatement, ImportStatement};

pub(crate) struct MultiQueryParse {
    pub async_tok: Option<Token!(!)>,
    pub client: Ident,
    pub args: Option<SdbArgs>,
    pub _arrow: FatArrow,
    pub _braces: Brace,
    pub stmts: Punctuated<SdbStatement, Token![;]>,
}

impl Parse for MultiQueryParse {
    fn parse(input: ParseStream) -> Result<Self> {
        let context;
        Ok(Self {
            async_tok: input.parse()?,
            client: input.parse()?,
            args: input.parse()?,
            _arrow: input.parse()?,
            _braces: braced!(context in input),
            stmts: context.parse_terminated(SdbStatement::parse)?,
        })
    }
}

impl MultiQueryParse {
    pub fn arg_steps(&self) -> TokenStream {
        match &self.args {
            Some(a) => a.field_assigns(),
            None => quote! {},
        }
    }

    pub fn prepare(&self, trans_db: &Ident) -> (TokenStream, TokenStream, TokenStream) {
        let result_handle = match self.async_tok {
            Some(_) => quote! { .unwrap() },
            None => quote! { ? },
        };

        let mut unpack = TokenStream::new();
        let mut push_steps = TokenStream::new();

        for line in &self.stmts {
            use SdbStatement as Ul;
            match line {
                Ul::Import( ImportStatement { source, _dollar, var_name, .. } ) => {
                    let var_str = LitStr::new(
                        &var_name.to_string(), 
                        var_name.span()
                    );
                    push_steps.extend(quote!( .push_var( #var_str, ( #source ) ) ))
                }

                Ul::ToVar {
                    sql,
                    _dollar,
                    var_name,
                    ..
                } => {
                    let var_str = LitStr::new(
                        &var_name.to_string(), 
                        var_name.span()
                    );
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
            }
        }

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

        for (line_num, line) in self.stmts.iter().enumerate() {
            match line {
                SdbStatement::Import( ImportStatement { var_name, .. } ) => {
                    vars.push((var_name.to_string(), line_num));
                },
                SdbStatement::ToVar { var_name, .. } => {
                    vars.push((var_name.to_string(), line_num));
                },
                _ => continue,
            }
        }
        vars
    }

    pub fn full_queries(&self) -> Vec<(&'_ LitStr, String)> {
        let mut queries = Vec::new();

        for line in self.stmts.iter() {
            match line {
                SdbStatement::Import { .. } => continue,
                SdbStatement::Ignored { ref sql }
                | SdbStatement::ToVar { ref sql, .. }
                | SdbStatement::Parse { ref sql, .. } => {
                    queries.push(
                        (&sql.literal, sql.complete_sql())
                    )
                }
            }
        }

        queries
    }
}

//

//

//


//
//
//



// pub(crate) struct MultiQueryParse {
//     pub client: Ident,
//     pub args: Option<SdbArgs>,
//     pub _arrow: FatArrow,
//     pub _braces: Brace,
//     pub stmts: Punctuated<SdbStatement, Token![;]>,
// }

// impl Parse for MultiQueryParse {
//     fn parse(input: ParseStream) -> Result<Self> {
//         let context;
//         let me = Self {
//             client: input.parse()?,
//             args: input.parse()?,
//             _arrow: input.parse()?,
//             _braces: braced!(context in input),
//             stmts: context.parse_terminated(SdbStatement::parse)?,
//         };

//         me.verify_formatting();

//         Ok(me)
//     }
// }

// impl MultiQueryParse {
//     pub fn arg_steps(&self) -> TokenStream {
//         match &self.args {
//             Some(a) => a.field_assigns(),
//             None => quote! {},
//         }
//     }

//     pub fn verify_formatting(&self) {
//         let mut tail_stmt = Option::<&SdbStatement>::None;
//         let mut parse_stmt = Option::<&SdbStatement>::None;

//         for (idx, line) in self.stmts.iter().enumerate() {
//             match line {
//                 SdbStatement::Parse { .. } => {
//                     parse_stmt = Some( line )
//                 },
//                 SdbStatement::ParseTail { .. } => {
//                     if let Some(first) = tail_stmt {
//                         return emit_error!(
//                             line, "Macro must only have one parsed statement";
//                             help = first => "First parsed statement"
//                         );
//                     }
//                     else if idx != self.stmts.len() - 1 { // tail must be the last one
//                         emit_error!(
//                             line, "Tail Statement must be at the end of the macro";
//                             help = "{}", STATEMENT_EXPL;
//                         );
//                     }
//                     else {
//                         tail_stmt = Some( line )
//                     }
//                 },
//                 _ => continue,
//             }
//         }

//         if tail_stmt.is_some() && parse_stmt.is_some() {
//             let tail = tail_stmt.unwrap();
//             let parse = parse_stmt.unwrap();
//             emit_error!(
//                 parse, "Macro doesn't support using assign and tail statements together";
//                 help = tail => "Tail statement";
//                 help = "{}", STATEMENT_EXPL;
//             );
//         }
//     }

//     pub fn prepare(&self, trans_db: &Ident) -> (TokenStream, TokenStream, TokenStream) {
//         let result_handle = match self.async_tok {
//             Some(_) => quote! { .unwrap() },
//             None => quote! { ? },
//         };

//         let mut unpack = TokenStream::new();
//         let mut push_steps = TokenStream::new();

//         for line in &self.stmts {
//             use SdbStatement as Ul;
//             match line {
//                 Ul::Import {
//                     source,
//                     _dollar,
//                     var_name,
//                     ..
//                 } => {
//                     let var_str = LitStr::new(
//                         &var_name.to_string(), 
//                         var_name.span()
//                     );
//                     push_steps.extend(quote!( .push_var( #var_str, ( #source ) ) ))
//                 }

//                 Ul::ToVar {
//                     sql,
//                     _dollar,
//                     var_name,
//                     ..
//                 } => {
//                     let var_str = LitStr::new(
//                         &var_name.to_string(), 
//                         var_name.span()
//                     );
//                     push_steps.extend(quote!( .query_to_var( #var_str, #sql ) ))
//                 }

//                 Ul::Ignored { sql } => push_steps.extend(quote!( .push_skipped( #sql ) )),

//                 Ul::Parse {
//                     sql,
//                     is_mut,
//                     store,
//                     path,
//                     ..
//                 } => {
//                     push_steps.extend(quote!( .push( #sql ) ));
//                     let call = path.call_next();
//                     unpack.extend(quote! {
//                         let #is_mut #store = #trans_db . #call #result_handle;
//                     })
//                 }

//                 Ul::ParseTail { sql, _as, path } => {
//                     push_steps.extend(quote!( .push( #sql ) ));
//                     let call = path.call_next();

//                     unpack = quote! { #call };
//                 }
//             }
//         }

//         (push_steps, unpack, result_handle)
//     }

//     pub fn get_tail(&self) -> Option<&'_ SdbStatement> {
//         for line in self.stmts.iter() {
//             if let SdbStatement::ParseTail { .. } = line {
//                 return Some( line )
//             }
//         }
//         None
//     }

//     pub fn arg_vars(&self) -> Vec<(String, usize)> {
//         let mut vars = Vec::new();

//         if let Some(args) = &self.args {
//             for i in 0..args.fields.len() {
//                 vars.push((format!("{i}"), 0))
//             }
//             for name in args.arg_names() {
//                 vars.push((name, 0))
//             }
//         }

//         for (line_num, line) in self.stmts.iter().enumerate() {
//             match line {
//                 SdbStatement::Import { var_name, .. } => {
//                     vars.push((var_name.to_string(), line_num));
//                 }
//                 SdbStatement::ToVar { var_name, .. } => {
//                     vars.push((var_name.to_string(), line_num));
//                 }
//                 _ => continue,
//             }
//         }
//         vars
//     }

//     pub fn full_queries(&self) -> Vec<(&'_ LitStr, String)> {
//         let mut queries = Vec::new();

//         for line in self.stmts.iter() {
//             match line {
//                 SdbStatement::Import { .. } => continue,
//                 SdbStatement::Ignored { ref sql }
//                 | SdbStatement::ToVar { ref sql, .. }
//                 | SdbStatement::Parse { ref sql, .. }
//                 | SdbStatement::ParseTail { ref sql, .. } => {
//                     queries.push(
//                         (&sql.literal, sql.complete_sql())
//                     )
//                 }
//             }
//         }

//         queries
//     }
// }
