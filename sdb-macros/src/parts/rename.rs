use syn::{parse::*, token::*, *};

pub(crate) struct TransFuncRename {
    pub _as: syn::token::As,
    pub ident: Ident,
}

impl TransFuncRename {
    #[allow(unused)]
    pub fn new_name(&self) -> &Ident {
        &self.ident
    }
}

impl Parse for TransFuncRename {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            _as: input.parse()?,
            ident: input.parse()?,
        })
    }
}

impl CustomToken for TransFuncRename {
    fn peek(cursor: buffer::Cursor) -> bool {
        if !parsing::peek_keyword(cursor, "as") {
            return false;
        }
        if let Some((_ident, _cur)) = cursor.ident() {
            true
        } else {
            false
        }
    }

    fn display() -> &'static str {
        todo!(";;;;")
    }
}
