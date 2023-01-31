use std::fmt::{Debug, Formatter, Result};

pub struct SqlErrorPointer<'a> {
    sql: &'a str,
    arrow_line: String,
}

impl<'a> SqlErrorPointer<'a> {
    pub fn new(sql: &'a str) -> Self {
        let mut arrow_line = String::with_capacity(sql.len());
        arrow_line.push_str("  ");

        Self { sql, arrow_line }
    }

    pub fn tick(mut self, space: usize, arrow: usize) -> Self {
        let start = self.arrow_line.chars().count() - 2;
        for _ in start..space {
            self.arrow_line.push('\u{00a0}');
        }
        for _ in 0..arrow {
            self.arrow_line.push('^');
        }
        self
    }
}

impl<'a> Debug for SqlErrorPointer<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "\n\n> {}\n{}\n\n", self.sql, self.arrow_line)
    }
}
