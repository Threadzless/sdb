use ::proc_macro2::Span;
use ::quote::ToTokens;
use ::syn::{parse::*, *};


pub(crate) struct ImportStatement {
    pub source: ExprBlock,
    pub _arrow: Token![=>],
    pub _dollar: Token![$],
    pub var_name: Ident,
}

impl Parse for ImportStatement {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            source: input.parse()?,
            _arrow: input.parse()?,
            _dollar: input.parse()?,
            var_name: input.parse()?,
        })
    }
}

impl ToTokens for ImportStatement {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.source.to_tokens(tokens);
        self._arrow.to_tokens(tokens);
        self._dollar.to_tokens(tokens);
        self.var_name.to_tokens(tokens);
    }
}

impl From<&ImportStatement> for Span {
    fn from(value: &ImportStatement) -> Self {
        value.source.block.brace_token.span
            .join( value.var_name.span() )
            .unwrap_or( value.source.block.brace_token.span )
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
