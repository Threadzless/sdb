use std::{iter::Peekable, str::CharIndices, fmt::Debug};

use super::*;

pub enum TokenTree<'c> {
    Whitespace {
        chars: &'c str,
        span: Span,
    },
    Group {
        chars: &'c str,
        borders: GroupBorder,
        start_idx: usize,
        end_idx: usize,
        inner: Vec<TokenTree<'c>>,
        span: Span,
    },
    Ident {
        chars: &'c str,
        span: Span,
    },
    Punct {
        chars: &'c str,
        span: Span,
    },
    Misk {
        chars: &'c str,
        span: Span,
    },
}


impl<'c> TokenTree<'c> {
    pub(crate) fn parse<'a>(s: &'c str, buff: &mut Peekable<CharIndices<'a>>) -> Result<Self, ()> {
        let Some((idx, ch)) = buff.peek() else { return Err( () ) };
        let start = *idx;
        let ch = *ch;
        match ch {
            // Whitespace chars
            '\u{09}' | '\u{0c}' | '\u{0a}' | '\u{0d}' | '\u{20}' => {
                Ok(Self::parse_whitespace(s, buff, start))
            },
            
            // Non-punctuation chars
            'a'..='z' | 'A'..='Z' | '_' => {
                Ok(Self::parse_ident(s, buff, start))
            },
            
            // Group Open
            '{' | '[' | '(' | '<' => {
                let border = GroupBorder::from_char(ch).unwrap();
                Ok(Self::parse_group(s, buff, start, border))
            },

            // Group Close 
            '}' | ']' | ')' | '>' => todo!(),

            // Group Toggle
            '\'' | '\"' | '|' | '/' => todo!(),

            // Punctuation chars
            '!'..='/' | ':'..='@' | '['..='`' | '{'..='~' => {
                Ok(Self::parse_punct(s, buff, start))
            },
            
            // Misc chars
            _ => todo!(),
        }
    }

    pub fn parse_whitespace(
        s: &'c str,
        buff: &mut Peekable<CharIndices<'_>>,
        start: usize
    ) -> Self
    {
        let mut end = start;
        while let Some((idx, ch)) = buff.peek() {
            if ch.is_ascii_whitespace() {
                end = *idx;
                buff.next();
            }
            else {
                break;
            }
        }

        Self::Whitespace {
            chars: &s[start..=end],
            span: Span(start..=end),
        }
    }

    pub fn parse_punct(
        s: &'c str,
        buff: &mut Peekable<CharIndices<'_>>,
        start: usize
    ) -> Self
    {
        let mut end = start;
        while let Some((idx, ch)) = buff.peek() {
            match ch {
                '!'..='/' | ':'..='@' | '['..='`' | '{'..='~' => {
                    end = *idx;
                    buff.next();
                },
                _ => break
            }
        }

        Self::Punct {
            chars: &s[start..=end],
            span: Span(start..=end),
        }
    }

    pub fn parse_ident(
        s: &'c str,
        buff: &mut Peekable<CharIndices<'_>>,
        start: usize
    ) -> Self
    {
        buff.next();
        let mut end = start;
        while let Some((idx, ch)) = buff.peek() {
            match *ch {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                    end = *idx;
                    buff.next();
                },
                _ => break
            }
        }

        Self::Ident {
            chars: &s[start..=end],
            span: Span(start..=end),
        }
    }

    pub fn parse_group(
        s: &'c str,
        buff: &mut Peekable<CharIndices<'_>>,
        start: usize,
        bounds: GroupBorder,
    ) -> Self
    {
        let start = start;
        let mut end = start;
        buff.next();

        let mut inner = vec![ ];
        let close_char = bounds.close().unwrap();
        
        while let Some((idx, ch)) = buff.peek() {
            end = *idx;
            if close_char.eq(ch) {
                end = *idx;
                buff.next();
                break;
            }
            if let Ok( tok ) = Self::parse(s, buff) {
                inner.push(tok)
            }
        }

        end += 2;
        
        if let Some(TokenTree::Punct { ref mut chars, ref mut span } ) = inner.last_mut() {
            if let Some(last) = chars.chars().last() && close_char.eq(&last) {
                let sp = span.0.clone();
                *chars = &chars[0..chars.len()-1];
                *span = Span( *sp.start()..=(*sp.end()-1) );
            }
        }

        Self::Group {
            inner,
            chars: &s[start..=end],
            span: Span(start..=end),
            borders: bounds,
            start_idx: start,
            end_idx: end,
        }
    }
}


impl Debug for TokenTree<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Whitespace { span, chars } => {
                write!(f, "White\t{chars:?}  {span:?}" )
            },
            Self::Group { borders, start_idx, end_idx, inner, span, .. } => {
                f.debug_struct("Group ")
                    .field("borders", borders)
                    .field("start_idx", start_idx)
                    .field("end_idx", end_idx)
                    .field("inner", inner)
                    .field("span", span)
                    .finish()
            },
            Self::Ident { span, chars } => {
                write!(f, "Ident\t{chars:?} {span:?}" )
            },
            Self::Punct { span, chars } => {
                write!(f, "Punct\t{chars:?} {span:?}" )
            },
            Self::Misk { span, chars } => {
                write!(f, "Misk \t{chars:?} {span:?}" )
            },
        }
    }
}