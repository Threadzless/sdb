// use anyhow:;
use serde::Deserialize;
use serde_json::{from_value, Value};

use crate::{error::SdbError, transaction::TransQuery};

use super::QueryReply;


/// The result of one entire SurrealDB transaction. Queries are grouped into 
/// transactions, even if you only use one.
pub struct TransactionReply {
    pub(crate) index: usize,
    pub(crate) queries: Vec<TransQuery>,
    pub(crate) replies: Vec<QueryReply>,
}

impl TransactionReply {
    pub(crate) fn new(queries: Vec<TransQuery>, mut replies: Vec<QueryReply>) -> Self {
        let mut idx = 0;
        for q in queries.iter() {
            if ! q.skip {
                replies.get_mut(idx).unwrap().query = Some( q.sql.clone() );
                idx += 1;
            }
        }

        Self {
            index: 0,
            queries,
            replies,
        }
    }

    fn next(&mut self) -> Result<(Value, &TransQuery), SdbError> {
        let reply = self.next_result().result.take();
        let idx = self.index - 1;
        let query = self.queries.get( self.index - 1 )
            .expect("Too many calls to TransactionReply::next_*.");

            println!("\t{}\t=> {}", query.skip, query.sql);

            Ok((reply, query))
    }


    pub fn next_result(&mut self) -> &mut QueryReply {
        while let Some( line ) = self.queries.get( self.index ) && line.skip {
            self.index += 1;
        }

        let reply = self.replies.get_mut(self.index)
            .expect("Too many calls to TransactionReply::next_*.");

        self.index += 1;
        reply
    }

    /// Get zero or more results
    pub fn next_list<T>(&mut self) -> Result<Vec<T>, SdbError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let result = self.next_result();

        match from_value::<Vec<T>>(result.result.take()) {
            Ok( v ) => Ok( v ),
            Err( err ) => {
                Err( SdbError::QueryResultParseFailure {
                    target_type: std::any::type_name::<Vec<T>>().to_string(),
                    serde_err: err,
                    query: result.query(),
                } )
            }
        }
    }

    /// Get zero or one results 
    pub fn next_one<T>(&mut self) -> Result<Option<T>, SdbError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let result = self.next_result();
        let Value::Array(mut arr) = result.result.take() else {
            panic!("Invalid input Transaction::next_option");
        };

        let Some( first ) = arr.get_mut( 0 ) else { return Ok( None ) };

        if first.is_null() { return Ok(None) };

        match from_value::<T>(first.take()) {
            Ok( v ) => Ok( Some( v ) ),
            Err( err ) => {
                Err( SdbError::QueryResultParseFailure {
                    target_type: std::any::type_name::<Option<T>>().to_string(),
                    serde_err: err,
                    query: result.query(),
                } )
            }
        }
    }

    /// Get exactly one result, or an error
    pub fn next_one_exact<T>(&mut self) -> Result<T, SdbError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let result = self.next_result();

        let mut vals: Vec<T> = result.parse();
        match vals.len() {
            0 => Err(SdbError::ZeroQueryResults {
                query: result.query(),
            }),
            _ => Ok(vals.remove(0)),
        }
    }
}
