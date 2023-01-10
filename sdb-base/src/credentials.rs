use base64::encode;
use std::fmt::Debug;

/// The information required to log into the SurrealDB instance.
///
/// TODO: implement Token and Digest methods
#[derive(Clone, Debug, PartialEq)]
pub enum Credentials {
    User { user: String },
    Basic { user: String, pass: String },
    // Token { token: String },
    // Digest {},
}

impl Credentials {
    pub fn auth_headers(&self) -> Vec<(String, String)> {
        let mut headers = Vec::new();
        let auth_str = match self {
            Credentials::User { user } => {
                let base = encode(format!("{user}:"));
                format!("Basic {base}")
            }
            Credentials::Basic { user, pass } => {
                let base = encode(format!("{user}:{pass}"));
                format!("Basic {base}")
            }
        };

        headers.push(("authorization".to_string(), auth_str));

        headers.push((
            "Access-Control-Allow-Credentials".to_string(),
            "true".to_string(),
        ));

        headers
    }
}
