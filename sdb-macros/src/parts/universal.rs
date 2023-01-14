// use proc_macro2::TokenStream;
// use quote::{quote, ToTokens};
use syn::{parse::*, token::*, *};

use crate::parts::*;

pub enum UniversalLine {
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

// impl UniversalLine {
//     #[deprecated]
//     pub fn get_sql( &self ) -> Option<&QuerySqlBlock> {
//         match self {
//             UniversalLine::Import { .. } => None,
//             UniversalLine::Ignored { sql } => Some( sql ),
//             UniversalLine::ToVar { sql, .. } => Some( sql ),
//             UniversalLine::Parse { sql, .. } => Some( sql ),
//             UniversalLine::ParseTail { sql, .. } => Some( sql ),
//         }
//     }
// }

impl Parse for UniversalLine {
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
//         todo!()
//     }
// }
