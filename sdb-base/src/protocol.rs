

    #[derive(Clone, Debug, PartialEq)]
    pub enum Protocol {
        /// Http POST requests. Slow, but ez.
        Http {
            secure: bool,
        },

        /// Websockets, faster.
        Socket {
            secure: bool,
        },

        /// TiKV - scalable distributed storage layer that's surrealDb compatible
        /// 
        /// Not implemented 
        Tikv {
            secure: bool,
        },
    }

    impl Protocol {
        pub fn prefix( &self ) -> &str {
            match self {
                Protocol::Http { secure: true } => "https",
                Protocol::Http { secure: false } => "http",
                Protocol::Socket { secure: true } => "wss",
                Protocol::Socket { secure: false } => "ws",
                _ => unimplemented!( ),
            }
        }

        pub fn parse( prefix: &str ) -> Option<Self> {
            match prefix {
                "ws" => Some( Protocol::Socket { secure: false } ),
                "wss" => Some( Protocol::Socket { secure: true } ),
                "http" => Some( Protocol::Http { secure: false } ),
                "https" => Some( Protocol::Http { secure: true } ),
                _ => None,
            }
        }

        pub fn is_secure( &self ) -> bool {
            match self {
                Protocol::Http { secure } => *secure,
                Protocol::Socket { secure } => *secure,
                Protocol::Tikv { secure } => *secure,
            }
        }
    }

    impl Default for Protocol {
        fn default() -> Self {
            Protocol::Socket { secure: true }    
        }
    }