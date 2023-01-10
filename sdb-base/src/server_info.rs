use crate::{
    credentials::Credentials,
    error::{SdbError, SdbResult},
    protocol::Protocol,
};

/// Describes how to connect to a surrealDB instance, including hostname,
/// namespace, dataspace, credentials, and protocol
/// 
/// ### Example
/// ```rust
/// # use sdb_base::prelude::*;
///
/// let connect_url = "wss://192.168.8.6:12345/test/demo";
/// let info = ServerInfo::new( connect_url, None, None ).unwrap();
///
/// assert_eq!(info.hostname, "192.168.8.6:12345");
/// assert_eq!(info.namespace, "test");
/// assert_eq!(info.database, "demo");
/// assert_eq!(info.protocol, Protocol::Socket { secure: true } );
/// assert_eq!(info.auth, None);
/// ```
/// 
/// 
#[derive(Clone, Debug, PartialEq)]
pub struct ServerInfo {
    pub hostname: String,
    pub namespace: String,
    pub database: String,
    pub protocol: Protocol,
    pub auth: Option<Credentials>,
}

impl ServerInfo {
    /// Creates a new [`ServerInfo`]
    /// 
    /// `host_string`: A URL-like string which describes how to connect to a SurrealDB 
    /// server. See below for syntax examples
    /// `protocol`: How to connect to the SurrealDB server. 
    /// 
    /// ### HostString Syntax
    /// Host strings are formatted just like a URL,
    ///  ```html
    /// [ <protocol>:// ] [ <username> [ : <password> ] @ ] <url_with_port> / <namespace> / <database>
    /// ```
    /// 
    /// ### Examples
    /// - `ws://test_user:test_pass@127.0.0.1:8934/test/demo`
    /// - `http://127.0.0.1:8000/example_ns/demo_db`
    /// - `wss://user_name_only@127.0.0.1/example_ns/demo_db`
    pub fn new(
        host_string: impl ToString,
        protocol: Option<Protocol>,
        auth: Option<Credentials>,
    ) -> SdbResult<Self> {
        let mut me = Self::inner_parse(&host_string.to_string())?;
        if auth.is_some() {
            me.auth = auth;
        }
        if let Some(p) = protocol {
            me.protocol = p;
        }

        Ok(me)
    }

    pub(crate) fn inner_parse(url: &str) -> Result<Self, SdbError> {
        let protocol;
        let main_url;
        match url.split_once("://") {
            Some((proto, rest)) => {
                main_url = rest;
                protocol = match Protocol::parse( proto ) {
                    Some(s) => s,
                    None => unimplemented!("Protocol {proto} not supported yet")
                };    
            },
            None => {
                main_url = url;
                protocol = Default::default();
            }
        }

        let parts = main_url.split("/").into_iter().collect::<Vec<&str>>();
        if parts.len() != 3 {
            return Err(SdbError::InvalidHostString {
                found: main_url.to_string()
            });
        }
        let mut auth = None;
        let mut parts = parts.iter();
        let mut host = *parts.next().unwrap();
        let ns = *parts.next().unwrap();
        let db = *parts.next().unwrap();

        if let Some( (left, right) ) = host.split_once("@") {
            if let Some( (user, pass) ) = left.split_once(":") {
                auth = Some( Credentials::Basic { 
                    user: user.to_string(),
                    pass: pass.to_string()
                });
                host = right;
            }
            else {
                unimplemented!("Non-user + pass authentication method")
            }
        }

        let con = ServerInfo {
            hostname: host.to_string(),
            namespace: ns.to_string(),
            database: db.to_string(),
            protocol,
            auth,
        };

        Ok(con)
    }

    /// Gets the URL of the surrealDb this [`ServerInfo`] points towards
    pub(crate) fn full_url(&self) -> String {
        let host = &self.hostname;
        let prefix = self.protocol.prefix();
        match &self.protocol {
            Protocol::Socket { .. } => format!("{prefix}://{host}/rpc"),
            Protocol::Http { .. } => format!("{prefix}://{host}/sql"),
            _ => unimplemented!(),
        }
    }

    /// Gets a list of headers for specifying the namespace, database, 
    /// and authentication method. 
    pub(crate) fn headers(&self) -> Vec<(String, String)> {
        #[cfg(feature = "log")]
        log::trace!("Generating connection headers");

        let mut heads = Vec::new();
        heads.push(("NS".to_string(), self.namespace.to_string()));
        heads.push(("DB".to_string(), self.database.to_string()));
        heads.push(("Accept".to_string(), "application/json".to_string()));

        if let Some(auth) = &self.auth {
            let mut auth_headers = auth.auth_headers();
            heads.append(&mut auth_headers);
        }

        heads
    }
}
