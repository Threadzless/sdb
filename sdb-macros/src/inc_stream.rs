use proc_macro2::{TokenStream, TokenTree};
// use quote::quote;

pub(crate) trait IncStream {
    fn extend_list<I>(&mut self, streams: I, seperator: TokenStream)
    where
        I: IntoIterator<Item = TokenTree>; //,( &mut self,  );
}

impl IncStream for TokenStream {
    fn extend_list<I>(&mut self, streams: I, seperator: TokenStream)
    where
        I: IntoIterator<Item = TokenTree>,
    {
        if !self.is_empty() {
            self.extend(seperator)
        }

        self.extend(streams)
    }
}
