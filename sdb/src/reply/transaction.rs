// use anyhow:;
use serde::Deserialize;
use serde_json::{from_value, Value};

use crate::{
    error::SdbError,
    transaction::TransQuery
};

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

    pub(crate) fn next_reply(&mut self) -> Result<&mut QueryReply, SdbError> {
        while let Some( line ) = self.queries.get( self.index ) && line.skip {
            self.index += 1;
        }

        let reply = self.replies.get_mut(self.index).expect("Too many calls to TransactionReply::next_*.");

        self.index += 1;
        Ok(reply)
    }

    fn next(&mut self) -> Result<Value, SdbError> {
        let reply = self.next_reply()?;
        Ok(reply.result.take())
    }

    pub fn next_vec<T>(&mut self) -> Result<Vec<T>, SdbError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let value = self.next()?;

        // #[cfg(feature = "log")]
        // log::info!(target: "sdb", "Extracting Vec<T>:\n {:?}", value );

        from_value::<Vec<T>>(value).map_err(|e| SdbError::Serde(e))
    }

    pub fn next_option<T>(&mut self) -> Result<Option<T>, SdbError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let value = self.next()?;

        let Value::Array(mut arr) = value else {
            panic!("Invalid input Transaction::next_option");
        };

        let Some( first ) = arr.get_mut( 0 ) else {
            return Ok( None )
        };

        if first.is_null() {
            return Ok( None )
        };

        match from_value::<T>( first.take() ) {
            Ok( v ) => Ok( Some( v ) ),
            Err(e) => Err(SdbError::Serde(e)),
        }
    }

    pub fn next_one<T>(&mut self) -> Result<T, SdbError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let value = self.next()?;

        // #[cfg(feature = "log")]
        // log::info!(target: "sdb", "Extracting T: {:?}", &value );

        // match &value {
        //     Value::Object( obj ) => {
        //         Ok( from_value::<T>(value)? )
        //     },
        //     Value::Array( arr ) => {
        //         if arr.len() > 0 {
        //             Ok( from_value::<Vec<T>>(value)?.remove(0) )
        //         }
        //         else {
        //             Ok()
        //             log::info!(target: "sdb", "B" );
        //             Err(SdbError::EndOfInput)
        //         }
        //     },
        //     _ => {
        //         log::info!(target: "sdb", "L" );
        //         Err(SdbError::EndOfInput)
        //     },
        // }

        // if value.is_array() {

        // }

        let mut vals = from_value::<Vec<T>>(value).unwrap();
        match vals.len() {
            0 => Err(SdbError::UnexpectedEndOfInput("Expected one result, but found zero".to_string())),
            _ => Ok(vals.remove(0)),
        }
    }
}
