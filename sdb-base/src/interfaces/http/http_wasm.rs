
use async_trait::async_trait;
use serde_json::from_str;
use gloo_net::http::{Request, Method, RequestCredentials, Headers};
use serde_json::Value;

use crate::{
    client::interface::*,
    reply::QueryReply,
    error::{SdbError, SdbResult},
    server_info::ServerInfo,
};


#[derive(Debug)]
pub struct HttpSurrealInterface {
}

impl SurrealInterfaceBuilder for HttpSurrealInterface {
    fn new(_info: &ServerInfo) -> SdbResult<Self> {
        Ok(Self { })
    }
}

impl HttpSurrealInterface {
    fn request(&self, info: &ServerInfo, sql: impl ToString) -> Result<Request, SdbError> {
        let head = Headers::new();
        for (k, v) in info.headers() {
            head.append(&k, &v)
        }

        let req = Request::new(&info.full_url())
            .headers(head)
            .method(Method::POST)
            .credentials( RequestCredentials::Include )
            .body(sql.to_string());

        Ok(req)
    }
}

#[async_trait]
impl SurrealInterface for HttpSurrealInterface {
    
    async fn send(&mut self, info: &ServerInfo, request: SurrealRequest) -> SdbResult<SurrealResponse> {
        let Some( Value::String( sql ) ) = request.params.get(0) else { panic!() };
        let req = self.request(info, &sql)?;

        let res = match req.send().await {
            Ok(res) => res,
            Err(err) => panic!("Netork Error: {err:?}"),
        };

        // todo!()

        let headers = res.headers();

        for (key, val) in headers.entries() {
            log::info!("    {} => {}", key, val );
        }

        // header_check("content-type", "application/json", headers.get("content-type"))?;
        // header_check("server", "SurrealDB", headers.get("server"))?;

        match res.text().await {
            Ok( text ) => {
                match from_str::<Vec<QueryReply>>( &text ) {
                    Ok( r ) => {
                        Ok( SurrealResponse::Result {
                            id: request.id,
                            result: Some( r )
                        })
                    },
                    Err( e ) => Err( SdbError::Serde( e ) )
                }
            },
            _ => panic!("Invalid response"),
        }
    }
}





fn header_check( header: &str, expect: &str, found: Option<String> ) -> SdbResult<()> {
    match found {
        Some(ct) if ct.starts_with(expect) => {
            Ok( () )
        }
        Some(ct) => {
            Err(SdbError::InvalidHeader {
                header: header.to_string(),
                expected: expect.to_string(),
                found: ct
            })                    
        },
        None => {
            Err(SdbError::MissingHeader { 
                header: header.to_string(),
                expected: expect.to_string(),
            })                    
        }
    }
}