#[derive(Clone, PartialEq)]
pub enum Bracket {
    Open(char, usize),
    Close(char, usize),
}

impl Bracket {
    pub fn new(c: char, idx: usize) -> Option<Bracket> {
        match c {
            '{' | '[' | '(' => Some(Bracket::Open(c, idx)),
            '}' | ']' | ')' => Some(Bracket::Close(c, idx)),
            _ => None,
        }
    }

    pub fn matches(&self, right: char) -> bool {
        match self {
            Bracket::Open(left, _) => {
                matches!((left, right), ('(', ')') | ('{', '}') | ('[', ']'))
            }
            // Bracket::Close( right ) => todo!(),
            _ => unreachable!(),
        }
    }

    pub fn pos(&self) -> usize {
        match self {
            Bracket::Open(_, pos) => *pos,
            Bracket::Close(_, pos) => *pos,
        }
    }
}

pub fn brackets_are_balanced(sql: &str) -> Result<Vec<(usize, usize)>, (usize, usize)> {
    let mut regions: Vec<(usize, usize)> = vec![];
    let mut stack: Vec<Bracket> = vec![];

    for (idx, curr_char) in sql.chars().enumerate() {
        let Some( brack ) = Bracket::new(curr_char, idx) else { continue };
        match brack {
            Bracket::Open(_, _) => {
                stack.push(brack);
            },
            Bracket::Close(ch, right_idx) if let Some(left) = stack.pop() => {
                if ! left.matches(ch) {
                    return Err( (left.pos(), right_idx)  )
                }
                regions.push((left.pos(), right_idx))
            }
            Bracket::Close(_, right_idx) => {
                return Err( (0, right_idx) )
            }
        }
    }
    if let Some(last) = stack.last() {
        // Not all brackets are matched
        let right_idx = sql.chars().count();
        Err((last.pos(), right_idx))
    } else {
        // regions.push((0, sql.chars().count()));
        Ok(regions)
    }
}
