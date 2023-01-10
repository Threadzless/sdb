use serde::Deserialize;
use serde_json::{Value, Error as ParseError, from_value};

use crate::{
    record::SurrealRecord, record_link::RecordLink,
};

/// Represents any type which can be stored in and extracted from a SurrealDB.
pub trait SurrealParseTarget: Sized + for<'de> Deserialize<'de> {
    fn parse( val: Value ) -> Result<Self, ParseError>;
}

impl<Trg: SurrealRecord> SurrealParseTarget for Trg {
    fn parse( val: Value ) -> Result<Self, ParseError> {
        serde_json::from_value( val )
    }
}

impl SurrealParseTarget for serde_json::Value {
    fn parse( val: Value ) -> Result<Self, ParseError> {
        Ok( val )
    }
}



impl<Link: SurrealRecord> SurrealParseTarget for RecordLink<Link> {
    fn parse( val: Value ) -> Result<Self, ParseError> {
        from_value( val )
    }
}

impl<Trg: SurrealParseTarget> SurrealParseTarget for Option<Trg> {
    fn parse( val: Value ) -> Result<Self, ParseError> {
        match val {
            Value::Null => Ok( None ),
            Value::Array(arr) if arr.len() == 0 => Ok( None ),
            Value::Array(mut arr) if arr.len() != 0 => {
                Ok( Some( Trg::parse( arr.remove(0) )? ) )
            },
            Value::Object(_) => {
                Ok( Some( Trg::parse( val )? ) )
            },
            _ => unimplemented!()
        }
    }
}

impl<Trg: SurrealParseTarget> SurrealParseTarget for Vec<Trg> {
    fn parse( val: Value ) -> Result<Self, ParseError> {
        match val {
            Value::Null => Ok( Vec::new() ),
            Value::Array(_) => serde_json::from_value( val ),
            _ => unimplemented!()
        }
    }
}
// macro_rules! impl_parse_target {
//     ($T: ty) => {
//         impl SurrealParseTarget for $T {
//             fn parse( val: Value ) -> Result<Self, ParseError> {
//                 from_value(val)
//             }
//         }
//     };
//     ($T1: ty, $T2: ty) => {
//         impl_parse_target!( $T1 );
//         impl_parse_target!( $T2 );
//     };
// }

// impl_parse_target!(String);
// impl_parse_target!(char);
// impl_parse_target!(bool);
// impl_parse_target!(u8, i8);
// impl_parse_target!(u16, i16);
// impl_parse_target!(u32, i32);
// impl_parse_target!(u64, i64);
// impl_parse_target!(u128, i128);
// impl_parse_target!(usize, isize);
// impl_parse_target!(f32, f64);

impl SurrealParseTarget for String {
    fn parse( val: Value ) -> Result<Self, ParseError> {
        match val {
            Value::String( s ) => Ok( s ),
            Value::Array( mut arr ) if arr.len() != 0 => {
                from_value( arr[0].take() )
            },
            mut v => from_value( v.take() )
        }
    }
}

impl SurrealParseTarget for f32 {
    fn parse( val: Value ) -> Result<Self, ParseError> {
        match val {
            Value::Number( n ) if let Some( n ) = n.as_f64() => Ok( n as Self ),
            Value::Array( mut arr ) if arr.len() != 0 => {
                from_value( arr[0].take() )
            },
            mut v => from_value( v.take() )
        }
    }
}

impl SurrealParseTarget for f64 {
    fn parse( val: Value ) -> Result<Self, ParseError> {
        match val {
            Value::Number( n ) if let Some( n ) = n.as_f64() => Ok( n as Self ),
            Value::Array( mut arr ) if arr.len() != 0 => {
                from_value( arr[0].take() )
            },
            mut v => from_value( v.take() )
        }
    }
}

macro_rules! impl_parse_target_numbers {
    ( $TU: ty, $TS: ty ) => {
        impl SurrealParseTarget for $TS {
            fn parse( val: Value ) -> Result<Self, ParseError> {
                match val {
                    Value::Number( n ) if let Some( n ) = n.as_i64() => Ok( n as Self ),
                    Value::Array( mut arr ) if arr.len() != 0 => {
                        from_value( arr[0].take() )
                    },
                    mut v => from_value( v.take() )
                }
            }
        }

        impl SurrealParseTarget for $TU {
            fn parse( val: Value ) -> Result<Self, ParseError> {
                match val {
                    Value::Number( n ) if let Some( n ) = n.as_u64() => Ok( n as Self ),
                    Value::Array( mut arr ) if arr.len() != 0 => {
                        from_value( arr[0].take() )
                    },
                    mut v => from_value( v.take() )
                }
            }
        }
    };
}

impl_parse_target_numbers!(u8, i8);
impl_parse_target_numbers!(u16, i16);
impl_parse_target_numbers!(u32, i32);
impl_parse_target_numbers!(u64, i64);
impl_parse_target_numbers!(u128, i128);
impl_parse_target_numbers!(usize, isize);
