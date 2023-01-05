use crate::{
    protocols::Protocol,
    credentials::Credentials,
    error::{SdbError, SdbResult},
};

/// Describes how to connect to a surrealDB instance, including hostname,
/// namespace, dataspace, credentials, and protocol
/// 
/// ```rust
/// # use sdb_base::prelude::*;
/// 
/// let info = ServerInfo::new( "ws://192.168.8.6:12345/test/demo", None, None );
/// 
/// assert_eq!(info.hostname, "192.168.8.6");
/// assert_eq!(info.port, Some(12345));
/// assert_eq!(info.namespace, "test");
/// assert_eq!(info.database, "demo");
/// assert_eq!(info.protocol, Some(Protocol::Socket));
/// assert_eq!(info.auth, None);
/// ```
/// 
#[derive(Clone, Debug)]
pub struct ServerInfo {
    pub hostname: String,
    pub port: Option<u16>,
    pub namespace: String,
    pub database: String,
    pub protocol: Protocol,
    pub auth: Option<Credentials>,
}

impl ServerInfo {
    /// 
    pub fn new(conn_string: impl ToString, protocol: Option<Protocol>, auth: Option<Credentials>) -> SdbResult<Self> {
        let mut me = Self::inner_parse(&conn_string.to_string())?;
        if me.auth.is_none() {
            me.auth = auth;
        }
        if let Some( p ) = protocol {
            me.protocol = p;
        }
        
        Ok( me )
    }

    pub fn full_url( &self ) -> String {
        let host = &self.hostname;
        match (&self.protocol, self.port) {
            (Protocol::Socket, Some(port)) => format!("ws://{host}:{port}/rpc"),
            (Protocol::Http, Some(port)) => format!("http://{host}:{port}/sql"),
            (Protocol::Socket, None) => format!("ws://{host}/rpc"),
            (Protocol::Http, None) => format!("http://{host}/sql"),
            _ => unimplemented!()
        }
    }

    pub(crate) fn inner_parse(url: &str) -> Result<Self, SdbError> {
        let protocol;
        let main_url;
        if let Some((proto, rest)) = url.split_once("://") {
            main_url = rest;
            protocol = match proto {
                #[cfg(feature = "ws")]
                "ws" | "wss" => Protocol::Socket,

                #[cfg(feature = "http")]
                "http" | "https" => Protocol::Http,

                #[cfg(feature = "tikv")]
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
        if parts.len() < 3 {
            return Err(SdbError::InvalidHostString( "Not enough slashes".to_string() ));
        }
        else if parts.len() > 3 {
            return Err(SdbError::InvalidHostString( "Too many slashes".to_string() ));
        }
        let mut parts = parts.iter();
        let mut host = *parts.next().unwrap();
        let mut port = None;
        let ns = *parts.next().unwrap();
        let db = *parts.next().unwrap();

        if let Some((domain, port_str)) = host.split_once(":") {
            host = domain;
            port = Some(
                u16::from_str_radix(port_str, 10)
                    .map_err(|_| SdbError::InvalidHostString( url.to_string() ))?
            )
        }

        let con = ServerInfo {
            hostname: host.to_string(),
            namespace: ns.to_string(),
            database: db.to_string(),
            port,
            protocol,
            auth: None
        };

        Ok(con)
    }

    pub fn headers( &self ) -> Vec<(String, String)> {
        #[cfg(feature = "log")]
        log::trace!("Generating connection headers");

        let mut heads = Vec::new();
        heads.push(("NS".to_string(), self.namespace.to_string()));
        heads.push(("DB".to_string(), self.database.to_string()));
        heads.push(("Accept".to_string(), "application/json".to_string()));

        if let Some( auth ) = &self.auth {
            let mut auth_headers = auth.auth_headers();
            heads.append( &mut auth_headers );
        }

        heads
    }
}
