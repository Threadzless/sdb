use serde::Deserialize;

use crate::{error::*, transaction::TransQuery};

use super::StatementResult;

/// The result of one entire SurrealDB transaction. Queries are grouped into
/// transactions, even if you only use one.
pub struct QueryReply {
    pub(crate) index: usize,
    pub(crate) queries: Vec<TransQuery>,
    pub(crate) replies: Vec<StatementResult>,
}

impl QueryReply {
    pub(crate) fn new(queries: Vec<TransQuery>, mut replies: Vec<StatementResult>) -> Self {
        let mut idx = 0;
        for q in queries.iter() {
            replies.get_mut(idx).unwrap().query = Some(q.sql.clone());
            idx += 1;
        }

        Self {
            index: 0,
            queries,
            replies,
        }
    }

    pub fn next_result<'a>(&'a mut self) -> &'a mut StatementResult {
        while let Some( line ) = self.queries.get( self.index ) && line.skip {
            self.index += 1;
        }

        let reply = self
            .replies
            .get_mut(self.index)
            .expect("Too many calls to TransactionReply::next_*.");

        #[cfg(feature = "log")]
        log::debug!("> {:?}\n", reply);

        println!("> {reply:?}\n");

        self.index += 1;
        reply
    }

    /// Get zero or more results
    pub fn next_vec<T>(&mut self) -> Result<Vec<T>, SdbError>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.next_result().parse_vec()
    }

    /// Get zero or one results
    pub fn next_opt<T>(&mut self) -> Result<Option<T>, SdbError>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.next_result().parse_opt()
    }

    /// Get exactly one result, or an error
    pub fn next_one<T>(&mut self) -> Result<T, SdbError>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.next_result().parse_one()
    }
}
