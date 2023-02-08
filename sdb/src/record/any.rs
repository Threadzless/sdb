use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Value, Map, Number};

use crate::record::{RecordId, SurrealRecord};

/// A Generic record which can hold any value. It's basically
/// just [`Value::Object`](serde_json::Value) but it implements
/// [`SurrealRecord`]
#[derive(Clone, Debug, PartialEq)]
pub struct AnyRecord {
    id: RecordId,
    fields: Value,
}

impl AnyRecord {
    /// Creates a new record intended for a given table, but without an Id.
    pub fn new( table: &str ) -> Self {
        Self::with_id( RecordId::placeholder(table) )
    }

    pub fn with_id( id: RecordId ) -> Self {
        let mut fields = Map::new();
        fields.insert("id".to_string(), Value::String( id.to_string() ) );

        Self {
            id,
            fields: Value::Object( fields ),
        }
    }

    pub fn take(&mut self, key: &str) -> Option<&mut Value> {
        self.fields.get_mut(key)
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.fields.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.fields.get_mut(key)
    }

    pub fn get_str(&self, key: &str) -> Option<&String> {
        match self.fields.get(key) {
            Some( Value::String( s ) ) => Some( s ),
            _ => None
        }
    }

    pub fn get_num(&self, key: &str) -> Option<&Number> {
        match self.fields.get(key) {
            Some( Value::Number( n ) ) => Some( n ),
            _ => None
        }
    }

    pub fn get_num_f64(&self, key: &str) -> Option<f64> {
        match self.fields.get(key) {
            Some( Value::Number( n ) ) => n.as_f64(),
            _ => None
        }
    }

    pub fn get_num_i64(&self, key: &str) -> Option<i64> {
        match self.fields.get(key) {
            Some( Value::Number( n ) ) => n.as_i64(),
            _ => None
        }
    }

    pub fn get_num_u64(&self, key: &str) -> Option<u64> {
        match self.fields.get(key) {
            Some( Value::Number( n ) ) => n.as_u64(),
            _ => None
        }
    }

    pub fn has(&self, key: &str) -> bool {
        self.fields.get(key).is_some()
    }

    pub fn set<V: Serialize>(&mut self, key: &str, val: V) {
        let Value::Object( ref mut obj ) = self.fields else { unreachable!() };
        let v = serde_json::to_value( val ).unwrap();
        obj.insert(key.to_string(), v);
    }
}

impl Serialize for AnyRecord {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.fields.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AnyRecord {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let parsed = <Value as Deserialize>::deserialize(deserializer)?;
        let Value::Object( ref obj ) = parsed else { panic!() };
        let Some( Value::String( id_str ) ) = obj.get("id") else { panic!() };
        let id =RecordId::parse( id_str.clone() ).expect("A valid recordId");
        Ok( Self {
            id,
            fields: parsed
        } )
    }
}

impl SurrealRecord for AnyRecord {
    fn id(&self) -> &RecordId {
        &self.id
    }

    fn table_name(&self) -> String {
        self.id.table()
    }

    fn record_fields(&self) -> Map<String, Value> {
        let Value::Object( map ) = &self.fields else { panic!() };
        map.iter()
            .filter_map(|(key, val)|{
                match key.eq("id") {
                    true => Some( (key.clone(), val.clone()) ),
                    false => None,
                }
            })
            .collect::<Map<String, Value>>()
    }
}
