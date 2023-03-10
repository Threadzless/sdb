use crate::{
    client::SurrealClient, credentials::Credentials, error::SdbResult, interfaces, protocol::*,
    server_info::ServerInfo,
};

use super::SurrealInterfaceBuilder;

/// The info needed to build a [`SurrealClient`]
pub struct ClientBuilder {
    connect_str: String,
    protocol: Option<Protocol>,
    auth: Option<Credentials>,
}

impl ClientBuilder {
    pub(crate) fn new(connect_string: &str) -> ClientBuilder {
        ClientBuilder {
            connect_str: connect_string.to_string(),
            protocol: None,
            auth: None,
        }
    }

    /// Set the protocol. If not set, te default is HTTP requests.
    pub fn protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = Some(protocol);
        self
    }

    /// Authenticate with just a user name
    pub fn auth_user(mut self, user: impl ToString) -> Self {
        self.auth = Some(Credentials::User {
            user: user.to_string(),
        });
        self
    }

    /// Authenticate with username and password
    pub fn auth_basic(mut self, user: impl ToString, pass: impl ToString) -> Self {
        self.auth = Some(Credentials::Basic {
            user: user.to_string(),
            pass: pass.to_string(),
        });
        self
    }

    // /// Not implemented
    // pub fn auth_token(mut self, token: impl ToString) -> Self {
    //     self.auth = Some(Credentials::Token {
    //         token: token.to_string(),
    //     });
    //     self
    // }

    pub fn build(self) -> SdbResult<SurrealClient> {
        let server = ServerInfo::new(self.connect_str, self.protocol, self.auth)?;

        let proto = server.protocol.clone();

        if proto.is_secure() {
            unimplemented!("Secure connections")
        }

        match proto {
            #[cfg(feature = "ws")]
            Protocol::Socket { .. } => {
                let inter = interfaces::WSSurrealInterface::new(&server)?;
                SurrealClient::build( server, inter )
            }

            #[cfg(feature = "http")]
            Protocol::Http { .. } => {
                let inter = interfaces::HttpSurrealInterface::new(&server)?;
                SurrealClient::build( server, inter )
            }

            #[cfg(feature = "tikv")]
            Protocol::Tikv { .. } => {
                unimplemented!()
            }

            #[allow(unreachable_patterns)]
            _ => panic!("Protocol not recognised: {proto:?}\n Did you enable the feature?"),
        }
    }
}
