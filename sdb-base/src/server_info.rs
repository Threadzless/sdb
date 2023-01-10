use crate::{
    credentials::Credentials,
    error::{SdbError, SdbResult},
    protocols::Protocol,
};

/// Describes how to connect to a surrealDB instance, including hostname,
/// namespace, dataspace, credentials, and protocol
///
/// ```rust
/// # use sdb_base::prelude::*;
///
/// let connect_url = "ws://192.168.8.6:12345/test/demo";
/// let info = ServerInfo::new( connect_url, None, None ).unwrap();
///
/// assert_eq!(info.hostname, "192.168.8.6:12345");
/// assert_eq!(info.namespace, "test");
/// assert_eq!(info.database, "demo");
/// assert_eq!(info.protocol, Protocol::Socket);
/// assert_eq!(info.auth, None);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ServerInfo {
    pub hostname: String,
    pub namespace: String,
    pub database: String,
    pub protocol: Protocol,
    pub auth: Option<Credentials>,
}

impl ServerInfo {
    ///
    pub fn new(
        conn_string: impl ToString,
        protocol: Option<Protocol>,
        auth: Option<Credentials>,
    ) -> SdbResult<Self> {
        let mut me = Self::inner_parse(&conn_string.to_string())?;
        if auth.is_some() {
            me.auth = auth;
        }
        if let Some(p) = protocol{
            me.protocol = p;
        }

        Ok(me)
    }

    pub(crate) fn inner_parse(url: &str) -> Result<Self, SdbError> {
        let protocol;
        let main_url;
        if let Some((proto, rest)) = url.split_once("://") {
            main_url = rest;
            protocol = match proto {
                "ws" | "wss" => Protocol::Socket,
                "http" | "https" => Protocol::Http,
                "tikv" => Protocol::Tikv,
                _ => panic!(
                    "Unrecognised network protocol: {:?}. Maybe you forgot to enable the feature?",
                    proto
                ),
            }
        } else {
            main_url = url;
            protocol = Default::default();
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
        match &self.protocol {
            Protocol::Socket => format!("ws://{host}/rpc"),
            Protocol::Http => format!("http://{host}/sql"),
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
