use ::std::time::Duration;
use ::serde_json::{Value, from_value};
use ::serde::{
    de::{MapAccess, Visitor},
    Deserialize, Serialize,
};

use crate::prelude::{SdbError, SdbResult};

#[derive(Debug, Serialize)]
pub struct StatementResult {
    pub query: Option<String>,
    pub time: Duration,
    pub status: String,
    pub result: Value,
}

impl StatementResult {
    pub fn parse_vec<T: for<'de> Deserialize<'de>>(&mut self) -> SdbResult<Vec<T>> {
        match serde_json::from_value(self.result.clone()) {
            Ok(v) => Ok(v),
            Err(err) => Err(SdbError::parse_failure::<T>(&self, err)),
        }
    }

    pub fn parse_one<T: for<'de> Deserialize<'de>>(&mut self) -> SdbResult<T> {
        match &mut self.result {
            Value::Array(arr) if arr.len() > 0 => {
                match from_value::<T>(arr[0].clone()) {
                    Ok(v) => Ok(v),
                    Err(err) => Err(SdbError::parse_failure::<T>(&self, err)),
                }
            },
            _ => {
                match from_value::<T>(self.result.clone()) {
                    Ok(v) => Ok(v),
                    Err(err) => Err(SdbError::parse_failure::<T>(&self, err)),
                }
            },
        }
    }

    pub fn parse_opt<T: for<'de> Deserialize<'de>>(&mut self) -> SdbResult<Option<T>> {
        match &mut self.result {
            Value::Array(arr) if arr.len() == 0 => {
                Ok(None)
            },
            Value::Array(arr) => {
                match from_value::<T>(arr[0].clone()) {
                    Ok(v) => Ok(Some(v)),
                    Err(err) => Err(SdbError::parse_failure::<T>(&self, err)),
                }
            },
            _ => {
                Ok( from_value::<T>(self.result.clone()).ok() )
            },
        }
    }

    pub fn query(&self) -> String {
        self.query.as_ref().unwrap().clone()
    }
}

impl<'de> Deserialize<'de> for StatementResult {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(QueryResultVisitor)
    }
}

struct QueryResultVisitor;

impl<'de> Visitor<'de> for QueryResultVisitor {
    type Value = StatementResult;

    fn expecting(&self, _formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        _formatter.write_str("A Valid SurrealDB response")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut result = None;
        let mut status = None;
        let mut time = None;

        let mut detail: Option<String> = None;

        while let Some(k) = map.next_key::<String>()? {
            match k.as_str() {
                "result" => {
                    result = Some(map.next_value()?);
                }
                "status" => {
                    status = Some(map.next_value()?);
                }
                "time" => {
                    let time_val: String = map.next_value()?;
                    time = Some(parse_durration(time_val.as_str()));
                }
                "detail" => {
                    detail = map.next_value()?;
                }
                _ => {
                    println!(" Unknown {} => {:?}", k, map.next_value::<Value>()?);
                }
            }
        }

        if let Some( time ) = time &&
        let Some( result ) = result &&
        let Some( status ) = status {
            Ok(StatementResult {
                time,
                result,
                status,
                query: None,
            })
        } else if let Some(detail) = detail {
            panic!("Query Failed: {detail}")
        } else {
            Err(serde::de::Error::missing_field("detail"))
            // panic!("TODO: implement ")
        }
    }
}

fn parse_durration(s: &str) -> Duration {
    let float_str = &s[0..s.len() - 3];
    let float = float_str.parse::<f64>().unwrap();

    if s.ends_with("us") || s.ends_with("µs") {
        Duration::from_secs_f64(float / 1_000_000.0)
    } else if s.ends_with("ms") {
        Duration::from_secs_f64(float / 1_000.0)
    } else if s.ends_with('s') {
        Duration::from_secs_f64(float)
    } else if s.ends_with("ns") {
        Duration::from_secs_f64(float / 1_000_000_000.0)
    } else if s.ends_with("ps") {
        Duration::from_secs_f64(float / 1_000_000_000_000.0)
    } else {
        panic!("Unrecognized duration suffix: {s}")
    }
}
