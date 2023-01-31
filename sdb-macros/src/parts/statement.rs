// use ::proc_macro::MultiSpan;
use ::proc_macro2::TokenStream;
// use proc_macro2::TokenStream;
use ::quote::{quote, ToTokens};
use ::syn::{parse::*, token::*, *};

use crate::parts::*;

pub(crate) enum SdbStatement {
    Import {
        source: ExprBlock,
        _arrow: Token![=>],
        _dollar: Token![$],
        var_name: Ident,
    },
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
    ParseTail {
        sql: QuerySqlBlock,
        _as: Token![as],
        path: QueryResultType,
    },
}

impl Parse for SdbStatement {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Brace) {
            return Ok(Self::Import {
                source: input.parse()?,
                _arrow: input.parse()?,
                _dollar: input.parse()?,
                var_name: input.parse()?,
            });
        }

        let sql: QuerySqlBlock = input.parse()?;
        if input.peek(Token![;]) {
            return Ok(Self::Ignored { sql });
        }

        if let Ok(_as) = input.parse() {
            return Ok(Self::ParseTail {
                sql,
                _as,
                path: input.parse()?,
            });
        }

        let _arrow = input.parse()?;

        if let Some(dollar) = input.parse::<Option<Token![$]>>()? {
            Ok(Self::ToVar {
                sql,
                _arrow,
                _dollar: dollar,
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
    pub(crate) fn get_runner(&self) -> TokenStream {
        let SdbStatement::ParseTail { path, .. } = self else {
            unreachable!("get_runner called on a non-tail statement")
        };

        match path {
            QueryResultType::Option(ty) => quote!{
                . run_parse_opt::<#ty>()
            },
            QueryResultType::Single(ty) =>quote!{
                . run_parse_one::<#ty>()
            },
            QueryResultType::Vec(ty) => quote!{
                . run_parse_vec::<#ty>()
            },
        }
    }
}

impl ToTokens for SdbStatement {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let stream = match self {
            SdbStatement::Import { source, _arrow, _dollar, var_name } => quote!{
                #source #_arrow #_dollar #var_name
            },
            SdbStatement::ToVar { sql, _arrow, _dollar, var_name } => quote!{
                #sql #_arrow #_dollar #var_name   
            },
            SdbStatement::Ignored { sql } => {
                return sql.to_tokens(tokens);
            }
            SdbStatement::Parse { sql, _arrow, is_mut, store, _colon, path } => quote!{
                #sql #_arrow #is_mut #store #_colon #path
            },
            SdbStatement::ParseTail { sql, _as, path } => quote!{
                #sql #_as #path
            },
        };

        tokens.extend(stream)
    }
}

impl From<&SdbStatement> for proc_macro2::Span {
    fn from(value: &SdbStatement) -> Self {
        match value {
            SdbStatement::Parse { sql, path, .. } |
            SdbStatement::ParseTail { sql, path, .. } => {
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
            SdbStatement::Import { source, var_name, .. } => {
                source.block.brace_token.span
                    .join( var_name.span() )
                    .unwrap_or( source.block.brace_token.span )
            }
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
