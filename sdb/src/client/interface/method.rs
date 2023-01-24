use serde::{Serialize, Deserialize, };

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum RequestMethod {
    Ping,
    /// Retrieve the current auth record
    Info,
    /// Switch to a specific namespace and database
    Use,
    /// Signup to a specific authentication scope
    Signup,
    /// Signin as a root, namespace, database or scope user
    Signin,
    /// Invalidate the current authentication session
    Invalidate,
    /// Authenticate using an authentication token
    Authenticate,
    /// Kill a live query using a query id
    Kill,
    /// Setup a live query on a specific table
    Live,
    /// Specify a connection-wide parameter
    #[serde(alias = "set")]
    Let,
    /// Unset and clear a connection-wide parameter
    Unset,
    /// Select a value or values from the database
    Select,
    /// Create a value or values in the database
    Create,
    /// Update a value or values in the database using `CONTENT`
    Update,
    /// Update a value or values in the database using `MERGE`
    #[serde(alias = "change")]
    Merge,
    /// Update a value or values in the database using `PATCH`
    #[serde(alias = "modify")]
    Patch,
    /// Delete a value or values from the database
    Delete,
    /// Specify the output format for text requests
    Format,
    /// Get the current server version
    Version,
    /// Run a full SurrealQL query against the database
    Query,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_test() {
        for (meth, txt) in [
            ( RequestMethod::Ping, r#""ping""# ),
            ( RequestMethod::Info, r#""info""# ),
            ( RequestMethod::Use, r#""use""# ),
            ( RequestMethod::Signup, r#""signup""# ),
            ( RequestMethod::Signin, r#""signin""# ),
            ( RequestMethod::Invalidate, r#""invalidate""# ),
            ( RequestMethod::Authenticate, r#""authenticate""# ),
            ( RequestMethod::Kill, r#""kill""# ),
            ( RequestMethod::Live, r#""live""# ),
            ( RequestMethod::Let, r#""let""# ),
            ( RequestMethod::Unset, r#""unset""# ),
            ( RequestMethod::Select, r#""select""# ),
            ( RequestMethod::Create, r#""create""# ),
            ( RequestMethod::Update, r#""update""# ),
            ( RequestMethod::Merge, r#""merge""# ),
            ( RequestMethod::Patch, r#""patch""# ),
            ( RequestMethod::Delete, r#""delete""# ),
            ( RequestMethod::Format, r#""format""# ),
            ( RequestMethod::Version, r#""version""# ),
            ( RequestMethod::Query, r#""query""# ),
        ] {
            parse_test_one(meth, txt)
        }

        // test aliases
        let val = serde_json::from_str::<RequestMethod>( r#""set""# ).unwrap();
        assert_eq!(val, RequestMethod::Let);

        let val = serde_json::from_str::<RequestMethod>( r#""change""# ).unwrap();
        assert_eq!(val, RequestMethod::Merge);

        let val = serde_json::from_str::<RequestMethod>( r#""modify""# ).unwrap();
        assert_eq!(val, RequestMethod::Patch);
    }

    fn parse_test_one( meth: RequestMethod, txt: &str ) {
        let val = serde_json::to_string( &meth ).unwrap();
        assert_eq!(val, txt);
        let val = serde_json::from_str::<RequestMethod>( &val ).unwrap();
        assert_eq!(val, meth);
    }
}