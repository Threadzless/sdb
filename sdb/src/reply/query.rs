use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Serialize,
};
use serde_json::Value;
use std::time::Duration;

#[derive(Debug, Serialize)]
pub struct QueryReply {
    pub query: Option<String>,
    pub time: Duration,
    pub status: String,
    pub result: Value,
}

impl QueryReply {
    pub fn parse_vec<T: for<'de> Deserialize<'de>>(&mut self) -> Vec<T> {
        let res = self.result.clone();
        serde_json::from_value(res).unwrap()
    }

    pub fn parse_one<T: for<'de> Deserialize<'de>>(&mut self) -> T {
        self.parse_opt().unwrap()
    }

    pub fn parse_opt<T: for<'de> Deserialize<'de>>(&mut self) -> Option<T> {
        let Value::Array( mut arr ) = self.result.clone() else {
            panic!( "Invalid response: Expected array, found \n\n{:?}\n\n", self.result )
        };
        let Some( one ) = arr.first_mut() else { return None };
        if let Ok(one) = serde_json::from_value::<T>(one.clone()) {
            return Some(one);
        }


        if let Ok( val ) = serde_json::from_value::<T>(one.clone()) {
            return Some( val )
        }
        match one.take() {
            Value::Object(mut obj) if obj.keys().len() == 1 => {
                let (_, inner) = obj.iter_mut().next().unwrap();
                serde_json::from_value::<T>(inner.take()).ok()
            },
            _ => None,
        }
    }

    pub fn query(&self) -> String {
        self.query.as_ref().unwrap().clone()
    }
}

impl<'de> Deserialize<'de> for QueryReply {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(QueryResultVisitor)
    }
}

struct QueryResultVisitor;

impl<'de> Visitor<'de> for QueryResultVisitor {
    type Value = QueryReply;

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
                    result = Some(map.next_value().expect("# # # i"));
                }
                "status" => {
                    status = Some(map.next_value().expect("# # # h"));
                }
                "time" => {
                    let time_val: String = map.next_value().expect("# # # g");
                    time = Some(parse_durration(time_val.as_str()));
                }
                "detail" => {
                    detail = map.next_value().expect("# # # j");
                }
                _ => {
                    println!(" Unknown {} => {:?}", k, map.next_value::<Value>().unwrap());
                }
            }
        }

        if let Some( time ) = time &&
        let Some( result ) = result &&
        let Some( status ) = status {
            Ok(QueryReply {
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