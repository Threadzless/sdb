use std::fmt::{Display, Formatter, Result as FmtResult};

use ::proc_macro2::{TokenStream, Ident};
use proc_macro_error::emit_error;
use ::quote::{quote, ToTokens};
use ::syn::{parse::*, punctuated::Punctuated, token::*, buffer::Cursor, ExprTuple, Token, bracketed};

pub struct InsertParse {
    pub client: Ident,
    pub _arrow1: FatArrow,
    pub ignore: Option<Token![!]>,
    pub table: Ident,
    pub fields: ExprTuple,
    pub _arrow2: FatArrow,
    pub values: InsertValues,
    pub ret: Option<InsertReturn>,
}

impl Parse for InsertParse {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            client: input.parse()?,
            _arrow1: input.parse()?,
            ignore: input.parse()?,
            table: input.parse()?,
            fields: input.parse()?,
            _arrow2: input.parse()?,
            values: input.parse()?,
            ret: input.parse()?,
        })
    }
}

impl InsertParse {
    pub fn build_insert_sql( &self ) -> TokenStream {
        let fields = self.fields.to_token_stream().to_string();
        let field_count = self.fields.elems.len();
        let table_name = self.table.to_string();

        let mut sql_bld = match self.ignore.is_some() {
            true => quote!{
                use ::serde_json::to_string;
                let mut sql = format!("INSERT IGNORE INTO {} {} VALUES\n", #table_name, #fields); 
            },
            false => quote!{
                use ::serde_json::to_string;
                let mut sql = format!("INSERT INTO {} {} VALUES\n", #table_name, #fields);
            }
        };

        for (i, row) in self.values.rows.iter().enumerate() {
            if row.elems.len() != field_count {
                emit_error!(row, "Rows must all have the same number of elements")
            }

            if i != 0 {
                sql_bld.extend(quote!{ sql.push_str(",\n"); });
            }
            sql_bld.extend(quote!{
                sql.push('(');
            });
            for (idx, field) in row.elems.iter().enumerate() {
                if idx != 0 {
                    sql_bld.extend(quote!{ sql.push(','); });
                }
                sql_bld.extend(quote!{
                    sql.push_str(&to_string( &#field ).unwrap());
                })
            }
            sql_bld.extend(quote!{
                sql.push(')');
            });
        }

        // append the return statement
        match self.ret {
            Some( ref ret ) => {
                let ret_str = ret.to_string();
                sql_bld.extend(quote!{
                    sql.push_str(#ret_str);
                });
                
                // if let InsertReturn::Field { ref field, .. } = ret {
                //     let field_name = field.to_string();
                //     sql_bld = quote!{
                //         #sql_bld
                //         let sql = format!("SELECT * FROM (SELECT {} FROM ({}))", #field_name, sql);
                //     }
                // }
            },
            None => sql_bld.extend(quote!{
                sql.push_str(" RETURN NONE");
            }),
        };

        sql_bld
    }
}

//

//

//

pub struct InsertValues {
    pub brack: Bracket,
    pub rows: Punctuated<ExprTuple, Token![,]>,
}

impl Parse for InsertValues {
    fn parse(input: ParseStream) -> Result<Self> {
        let ctx;
        Ok(Self {
            brack: bracketed!(ctx in input),
            rows: ctx.parse_terminated(ExprTuple::parse)?,
        })
    }
}

impl ToTokens for InsertValues {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.brack.surround(tokens, |t| self.rows.to_tokens(t) )
    }
}

//

//

pub enum InsertReturn {
    Field {
        _return: Token![return],
        field: Ident,
    },
    Fields {
        _return: Token![return],
        fields: Punctuated<Ident, Token![,]>,
    },
    Star {
        _return: Token![return],
        _star: Token![*],
    }
}

impl Parse for InsertReturn {
    fn parse(input: ParseStream) -> Result<Self> {
        let _return = input.parse()?;

        if input.peek(Token![*]) {
            Ok(Self::Star { 
                _return,
                _star: input.parse()?,
            })
        }
        else {
            let fields: Punctuated<Ident, Token![,]> = input.parse_terminated(Ident::parse)?;
            if fields.len() == 1 {
                Ok(Self::Field {
                    _return,
                    field: fields.into_iter().next().unwrap(),
                })
            }
            else {
                Ok(Self::Fields { _return, fields })
            }
        }
    }
}

impl ToTokens for InsertReturn {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Fields { _return, fields } => {
                _return.to_tokens(tokens);
                fields.to_tokens(tokens);
            },
            Self::Field { _return, field } => {
                _return.to_tokens(tokens);
                field.to_tokens(tokens);
            },
            Self::Star { _return, _star } => {
                _return.to_tokens(tokens);
                _star.to_tokens(tokens);
            }
        }
    }
}

impl CustomToken for InsertReturn {
    fn peek(cursor: Cursor) -> bool {
        match cursor.ident() {
            Some((ident, _)) if ident.to_string().eq("return") => true,
            _ => false,
        }
    }

    fn display() -> &'static str {
        todo!()
    }
}

impl Display for InsertReturn {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, " RETURN ")?;
        match self {
            Self::Star { .. } => write!(f, "*")?,
            Self::Field { field, .. } => write!(f, "{}", field.to_string())?,
            Self::Fields { fields, .. } => {
                write!(f, "{}", fields.iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
                )?;
            },
        }
        Ok( () )
    }
}