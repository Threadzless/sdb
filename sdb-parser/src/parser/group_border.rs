
#[derive(Debug)]
pub enum GroupBorder {
    Parenthesies,
    Bracket,
    Braces,
    Triangle,
    None,
    BackTick,
    VLine,
    DoubleQuote,
    SingleQuote,
    // Other,
}

impl GroupBorder {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '{' | '}' => Some(Self::Braces),
            '[' | ']' => Some(Self::Bracket),
            '(' | ')' => Some(Self::Parenthesies),
            '<' | '>' => Some(Self::Triangle),
            '\'' => Some(Self::SingleQuote),
            '"' => Some(Self::DoubleQuote),
            '`' => Some(Self::BackTick),
            '|' => Some(Self::VLine),
            _ => None,
        }
    }

    pub fn open(&self) -> Option<char> {
        match self {
            GroupBorder::Parenthesies => Some('('),
            GroupBorder::DoubleQuote => Some('\"'),
            GroupBorder::SingleQuote => Some('\''),
            GroupBorder::Triangle => Some('<'),
            GroupBorder::BackTick => Some('`'),
            GroupBorder::Bracket => Some('['),
            GroupBorder::Braces => Some('{'),
            GroupBorder::VLine => Some('|'),
            _ => None,
        }
    }

    pub fn close(&self) -> Option<char> {
        match self {
            GroupBorder::Parenthesies => Some(')'),
            GroupBorder::DoubleQuote => Some('\"'),
            GroupBorder::SingleQuote => Some('\''),
            GroupBorder::Triangle => Some('>'),
            GroupBorder::BackTick => Some('`'),
            GroupBorder::Bracket => Some(']'),
            GroupBorder::Braces => Some('}'),
            GroupBorder::VLine => Some('|'),
            _ => None,
        }
    }

    pub fn nestable(&self) -> bool {
        match self {
            GroupBorder::Parenthesies |
            GroupBorder::Bracket |
            GroupBorder::Braces |
            GroupBorder::Triangle => true,
            _ => false,
        }
    }
}