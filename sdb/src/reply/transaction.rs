// use anyhow:;
use serde::Deserialize;
use serde_json::{from_value, Value};

use crate::{error::SdbError, transaction::TransQuery};

use super::QueryReply;

// use crate::TransQuery;

pub struct TransactionReply {
    pub(crate) index: usize,
    pub(crate) queries: Vec<TransQuery>,
    pub(crate) replies: Vec<QueryReply>,
}

impl TransactionReply {
    pub(crate) fn new(queries: Vec<TransQuery>, replies: Vec<QueryReply>) -> Self {
        Self {
            index: 0,
            queries,
            replies,
        }
    }

    pub(crate) fn next_reply(&mut self) -> Result<(&mut QueryReply, &TransQuery), SdbError> {
        while let Some( line ) = self.queries.get( self.index ) && line.skip {
            self.index += 1;
        }

        let reply = self.replies.get_mut(self.index)
            .expect("Too many calls to TransactionReply::next_*.");

        let query = self.queries.get(self.index)
            .expect("Too many calls to TransactionReply::next_*.");

        self.index += 1;
        Ok((reply, query))
    }

    fn next(&mut self) -> Result<(Value, &TransQuery), SdbError> {
        let (reply, query) = self.next_reply()?;
        Ok((reply.result.take(), query))
    }

    pub fn next_vec<T>(&mut self) -> Result<Vec<T>, SdbError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let (value, query) = self.next()?;

        match from_value::<Vec<T>>(value) {
            Ok( v ) => Ok( v ),
            Err( err ) => {
                Err( SdbError::QueryResultParseError {
                    parse_target: std::any::type_name::<Vec<T>>().to_string(),
                    serde_err: err,
                    query: query.sql.clone()
                } )
            }
        }
    }

    pub fn next_option<T>(&mut self) -> Result<Option<T>, SdbError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let (value, query) = self.next()?;

        let Value::Array(mut arr) = value else {
            panic!("Invalid input Transaction::next_option");
        };

        let Some( first ) = arr.get_mut( 0 ) else { return Ok( None ) };

        if first.is_null() { return Ok(None) };

        match from_value::<T>(first.take()) {
            Ok( v ) => Ok( Some( v ) ),
            Err( err ) => {
                Err( SdbError::QueryResultParseError {
                    parse_target: std::any::type_name::<Option<T>>().to_string(),
                    serde_err: err,
                    query: query.sql.clone()
                } )
            }
        }
    }

    pub fn next_one<T>(&mut self) -> Result<T, SdbError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let (value, _query) = self.next()?;

        let mut vals = from_value::<Vec<T>>(value).unwrap();
        match vals.len() {
            0 => Err(SdbError::UnexpectedEndOfInput(
                "Expected one result, but found zero".to_string(),
            )),
            _ => Ok(vals.remove(0)),
        }
    }
}
