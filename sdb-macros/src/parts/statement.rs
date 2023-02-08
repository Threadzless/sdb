use ::proc_macro2::Span;
use ::quote::{quote, ToTokens};
use ::syn::{parse::*, token::*, *};

use crate::parts::*;

mod import;
mod tail;

pub(crate) use import::*;
pub(crate) use tail::*;


pub(crate) enum SdbStatement {
    Import( ImportStatement ),
    ToVar {
        sql: QuerySqlBlock,
        _arrow: Token![=>],
        _dollar: Token![$],
        var_name: Ident,
    },
    Ignored {
        sql: QuerySqlBlock,
    },
    Parse {
        sql: QuerySqlBlock,
        _arrow: Token![=>],
        is_mut: Option<Token![mut]>,
        store: Ident,
        _colon: Token![:],
        path: QueryResultType,
    },
}

impl Parse for SdbStatement {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Brace) {
            return Ok(Self::Import( input.parse()? ));
        }

        let sql: QuerySqlBlock = input.parse()?;
        if input.peek(Token![;]) {
            return Ok(Self::Ignored { sql });
        }
        else if input.peek(Token![as]) {
            let msg = format!(
                "`\"SQL\" as TYPE` not supported in queries! macro. Use assign syntax instead:\n{}",
                "\"SQL\" => var_name: TYPE\n"
            );
            return Err( input.error(msg) )
        }

        let _arrow = input.parse()?;

        if let Some(_dollar) = input.parse::<Option<Token![$]>>()? {
            Ok(Self::ToVar {
                sql,
                _arrow,
                _dollar,
                var_name: input.parse()?,
            })
        } else {
            Ok(Self::Parse {
                sql,
                _arrow,
                is_mut: input.parse()?,
                store: input.parse()?,
                _colon: input.parse()?,
                path: input.parse()?,
            })
        }
    }
}

impl SdbStatement {
    // pub(crate) fn get_runner(&self) -> TokenStream {
    //     match self {
    //         SdbStatement::ParseTail( ref tail ) => tail.get_runner(),
    //         _ => unreachable!("get_runner called on a non-tail statement")
    //     }
    // }
}

impl ToTokens for SdbStatement {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            SdbStatement::Import( i ) => i.to_tokens(tokens),
            // SdbStatement::ParseTail( tail ) => tail.to_tokens(tokens),
            SdbStatement::ToVar { sql, _arrow, _dollar, var_name } => {
                tokens.extend(quote!{#sql #_arrow #_dollar #var_name })
            },
            SdbStatement::Ignored { sql } => {
                sql.to_tokens(tokens);
            }
            SdbStatement::Parse { sql, _arrow, is_mut, store, _colon, path } => {
                tokens.extend(quote!{ #sql #_arrow #is_mut #store #_colon #path })
            },
        }
    }
}

impl From<&SdbStatement> for Span {
    fn from(value: &SdbStatement) -> Self {
        match value {
            // SdbStatement::ParseTail( tail ) => {
            //     Span::from( tail )
            // },
            SdbStatement::Parse { sql, path, .. } => {
                sql.literal.span()
                    .join( path.span() )
                    .unwrap_or( sql.literal.span() )
            },
            SdbStatement::ToVar { sql, var_name, .. } => {
                sql.literal.span()
                    .join( var_name.span() )
                    .unwrap_or( sql.literal.span() )
            },
            SdbStatement::Ignored { sql } => {
                sql.literal.span()
            },
            SdbStatement::Import( i ) => Span::from(i),
        }
    }
}

// impl MultiSpan for &SdbStatement {
//     fn into_spans(self) -> Vec<proc_macro::Span> {
//         match self {
//             SdbStatement::ParseTail { sql, path, .. } => {
//                 let span = sql.literal.span()
//                     .join( path.span() )
//                     .unwrap_or( sql.literal.span() );
//                 vec![ span.unwrap() ]
//             },
//             _ => todo!("SdbStatement::span")
//         }
//     }
// }

// impl ToTokens for UniQueryLine {
//     fn to_tokens(&self, tokens: &mut TokenStream) {
//         let full_sql = &self.sql;
//         tokens.extend(quote! {
//             .push( #full_sql )
//         })
//     }
// }

// pub struct UniType {
//     pub colon: Token![:],
//     pub path: TypePath,
// }

// impl Parse for UniType {
//     fn parse(input: ParseStream) -> Result<Self> {
//         Ok(Self {
//             colon: input.parse()?,
//             path: input.parse()?,
//         })
//     }
// }

// impl ToTokens for UniType {
//     fn to_tokens(&self, tokens: &mut TokenStream) {
//         let colon = &self.colon;
//         let path = &self.path;
//         tokens.extend(quote!{ #colon #path })
//     }
// }

// impl CustomToken for UniType {
//     fn peek(cursor: buffer::Cursor) -> bool {
//         if let Some( (punct, _cur) ) = cursor.punct() {
//             punct.as_char().eq(&':')
//         }
//         else {
//             false
//         }
//     }

//     fn display() -> &'static str {
//         
//     }
// }
