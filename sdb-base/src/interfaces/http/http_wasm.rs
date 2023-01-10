
use serde_json::from_str;
use gloo_net::http::{Request, Method, RequestCredentials, Headers};
use serde_json::Value;

// use wasm_bindgen_futures::spawn_local;

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

unsafe impl Send for HttpSurrealInterface { } 
unsafe impl Sync for HttpSurrealInterface { }

#[async_trait::async_trait(?Send)]
impl SurrealInterface for HttpSurrealInterface {
    
    async fn send(&mut self, info: &ServerInfo, request: SurrealRequest) -> SdbResult<SurrealResponse> {

        let Some( Value::String( sql ) ) = request.params.get(0) else { panic!() };
        let req = self.request(info, &sql)?;

        let res = match req.send().await {
            Ok(res) => res,
            Err(err) => panic!("Netork Error: {err:?}"),
        };

        let headers = res.headers();

        for (key, val) in headers.entries() {
            log::info!("    {} => {}", key, val );
        }

        header_check("content-type", "application/json", headers.get("content-type"))?;
        header_check("server", "SurrealDB", headers.get("server"))?;

        let res_text = res.text();
        let res_text = res_text.await;
        match &res_text {
            Ok( text ) => {
                match from_str::<Vec<QueryReply>>( &text ) {
                    Ok( r ) => {
                        Ok( SurrealResponse::Result {
                            id: request.id,
                            result: Some( r )
                        })
                    },
                    Err( e ) => Err( SdbError::QueryResultParseFailure {
                        query: String::new(),
                        target_type: "Vec<QueryReply>".to_string(),
                        serde_err: e
                    } )
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
            Err(SdbError::ServerNotSurreal { 
                why: format!("Header {header:?} is supposed to be {expect:?}, but found {ct:?}")
            })                    
        },
        None => {
            Err(SdbError::ServerNotSurreal { 
                why: format!("Header {header:?} is supposed to be {expect:?}, but it is not set")
            })                    
        }
    }
}