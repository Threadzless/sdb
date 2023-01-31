use ::proc_macro2::{TokenStream, Ident};
use ::quote::ToTokens;
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

pub struct InsertReturn {
    pub _return: Token![return],
    pub expected: TokenStream,
}

impl Parse for InsertReturn {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            _return: input.parse()?,
            expected: input.parse()?,
        })
    }
}

impl ToTokens for InsertReturn {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self._return.to_tokens(tokens);
        self.expected.to_tokens(tokens);
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