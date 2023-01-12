// use proc_macro2::TokenStream;
// use quote::{quote, ToTokens};
// use syn::{parse::*, Ident, LitStr, Token};

// mod let_line;
// mod raw_line;
// mod select_anon;
// mod select_line;

// pub(crate) use let_line::*;
// pub(crate) use raw_line::*;
// // pub(crate) use select_anon::*;
// pub(crate) use select_line::*;



// pub enum QueryLine {
//     /// run a query, ignore the results
//     Raw(RawQueryLine),
//     /// Run a query and store the results in a transaction variable
//     Let(LetQueryLine),
//     /// Run a query then parse the results into a rust variable
//     Select(SelectQueryLine),
// }

// impl Parse for QueryLine {
//     fn parse(input: ParseStream) -> Result<Self> {
//         if input.peek(Token![$]) {
//             Ok(Self::Let(input.parse()?))
//         }
//         else if ! input.peek(LitStr) {
//             Ok(Self::Raw(input.parse()?))
//         } else {
//             Ok(Self::Select(input.parse()?))
//         }
//     }
// }

// impl ToTokens for QueryLine {
//     fn to_tokens(&self, tokens: &mut TokenStream) {
//         use QueryLine::*;
//         match self {
//             Raw(r) => r.to_tokens(tokens),
//             Let(l) => l.to_tokens(tokens),
//             Select(s) => s.to_tokens(tokens),
//         }
//     }
// }

// impl QueryLine {
//     pub fn out_call(&self, transact: &Ident, err: &TokenStream) -> Option<TokenStream> {
//         let Self::Select( select ) = self else { return None };

//         let into = &select.into;
//         let cast = &select.cast;
//         let call_next = cast.call_next();

//         Some(quote! {
//             let #into = #transact . #call_next () #err ;
//         })
//     }
// }
