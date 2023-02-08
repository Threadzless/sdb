use ::proc_macro2::{TokenStream, Span};
use ::quote::{quote, ToTokens};
use ::syn::{parse::*, Token};

use crate::parts::*;

pub(crate) struct TailStatement {
    pub sql: QuerySqlBlock,
    pub _as: Token![as],
    pub path: QueryResultType,
}

impl Parse for TailStatement {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            sql: input.parse()?,
            _as: input.parse()?,
            path: input.parse()?,
        })
    }
}

impl TailStatement {
    pub(crate) fn get_runner(&self) -> TokenStream {
        match &self.path {
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

impl ToTokens for TailStatement {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.sql.to_tokens(tokens);
        self._as.to_tokens(tokens);
        self.path.to_tokens(tokens);
    }
}

impl From<&TailStatement> for Span {
    fn from(value: &TailStatement) -> Self {
        value.sql.literal.span()
            .join( value.path.span() )
            .unwrap_or( value.sql.literal.span() )
    }
}
