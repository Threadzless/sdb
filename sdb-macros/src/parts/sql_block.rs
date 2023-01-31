use ::std::fmt::{Debug, Formatter, Result as FmtResult};
use ::proc_macro2::TokenStream;
use ::proc_macro_error::{emit_error, emit_warning};
use ::quote::{quote, ToTokens};
use ::syn::{parse::*, LitStr, Token};

use super::QuerySugar;

pub(crate) struct QuerySqlBlock {
    pub literal: LitStr,
    pub sugars: Vec<QuerySugar>,
}

impl Parse for QuerySqlBlock {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut me = Self {
            literal: input.parse()?,
            sugars: Vec::new(),
        };

        while input.peek(Token![.]) {
            me.sugars.push(input.parse::<QuerySugar>()?)
        }

        me.check();

        Ok(me)
    }
}

impl ToTokens for QuerySqlBlock {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let sql = self.complete_sql();
        let sql_lit = LitStr::new(&sql, self.literal.span());
        tokens.extend(quote!( #sql_lit ));
    }
}

impl Debug for QuerySqlBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("QuerySqlBlock")
            .field("methods", &self.sugars)
            .field("sql", &self.literal)
            .finish()
    }
}

impl QuerySqlBlock {
    /// Emits an error if the SQL string contains any semicolons, and
    /// a emits a warning if the query contains comments
    pub fn check(&self) {
        let sql_span = &self.literal.span();
        let sql_str = self.literal.value();

        if sql_str.contains("--") {
            emit_warning!( sql_span, "Queries shouldn't contain comments";
                help = "Remove doube minuses (--) from you query.\nComments make the syntax checker get all fucky wucky"
            )
        }

        let Some( (first, second) ) = sql_str.split_once(';') else { return };

        let suggestion = format!("{first:?};\n{second:?}");

        emit_error!( sql_span, "Queries cannot contain semicolons (;)";
            help = "Try splitting the query into seperate query strings:\n\n{}", suggestion;
        )
    }

    pub(crate) fn build_sugar(&self, mut sql: String) -> Option<String> {
        if self.sugars.len() == 0 {
            return None
        }

        for method in &self.sugars {
            method.apply_method_sql(&mut sql)
        }
        
        Some( sql )
    }

    pub fn complete_sql(&self) -> String {
        let sql = self.literal.value();

        match self.build_sugar(sql) {
            Some(full_sql) => full_sql,
            None => self.literal.value(),
        }
    }
}