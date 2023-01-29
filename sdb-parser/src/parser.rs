use std::{iter::Peekable, str::CharIndices};


mod group_border;
mod span;
mod tree;

pub use group_border::*;
pub use span::*;
pub use tree::*;


pub struct TreeParser<'a> {
    text: String,
    buff: Peekable<CharIndices<'a>>,
    tokens: Vec<TokenTree<'a>>
}

impl<'a> TreeParser<'a> {
    pub fn new( raw: impl ToString ) -> Self {
        let text = raw.to_string();
        let text_ptr = &text;
        let text_ptr = unsafe { &*(text_ptr as &String as *const String) };
        Self {
            text,
            buff: text_ptr.char_indices().peekable(),
            tokens: vec![]
        }
    }

    pub fn parse<'b: 'a>(&'b mut self) -> &'_ Vec<TokenTree> {
        let pbuff = unsafe { &mut *(&mut self.buff as &'b mut Peekable<CharIndices<'b>> as *mut Peekable<CharIndices<'b>>) };
        loop {
            let ptr = unsafe { &mut *(pbuff as &mut _ as *mut _ ) };
            if let Ok(tt) = TokenTree::parse(&self.text, ptr) {
                self.tokens.push( tt )
            }
            else {
                break;
            }
        }

        &self.tokens
    }
}
