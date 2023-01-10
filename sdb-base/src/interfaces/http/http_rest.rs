use reqwest::{Client, ClientBuilder, RequestBuilder, Url};
use serde_json::Value;

use crate::{
    server_info::ServerInfo,
    client::interface::*,
    // reply::QueryReply,
    error::{SdbError, SdbResult}, reply::QueryReply,
};


#[derive(Debug)]
pub struct HttpSurrealInterface {
    client: Client,
}

impl SurrealInterfaceBuilder for HttpSurrealInterface {
    fn new(_info: &ServerInfo) -> SdbResult<Self> {
        let client = ClientBuilder::new()
            .build()
            .unwrap();

        Ok(Self {
            client,
        })
    }
}

impl HttpSurrealInterface {
    // #[cfg(not( target_family = "wasm"))]
    fn request(&self, info: &ServerInfo, sql: impl ToString) -> Result<RequestBuilder, SdbError> {
        let url = Url::parse(&info.full_url())
            .expect("A valid hostname for the database");

        let mut req = self.client.post(url)
            .body(sql.to_string());

        for (k, v) in info.headers() {
            req = req.header(k, v);
        }

        Ok(req)
    }
}

#[async_trait::async_trait(?Send)]
impl SurrealInterface for HttpSurrealInterface {
    // fn send(&mut self, info: &ServerInfo, sql: String) -> Result<Vec<QueryReply>, SdbError> {
    async fn send(&mut self, info: &ServerInfo, request: SurrealRequest) -> SdbResult<SurrealResponse> {
        let Some( Value::String( sql ) ) = request.params.get(0) else { unreachable!() };
        let req = self.request(info, sql)?;
        let res = convert_err( req.send().await, info )?;
        let txt = convert_err( res.text().await, info )?;
        match serde_json::from_str::<Vec<QueryReply>>( &txt ) {
            Err( _err ) => unreachable!("Response Parse Failure"),
            Ok( replies ) => Ok(
                SurrealResponse::Result {
                    id: request.id, 
                    result: Some( replies) 
                }
            )
        }
    }
}

fn convert_err<T>(
    base: Result<T, reqwest::Error>, 
    info: &ServerInfo
) -> Result<T, SdbError> {
    let base = match base {
        Err( e ) => e,
        Ok( val ) => return Ok( val )
    };

    let url = info.full_url();

    if base.is_timeout() {
        Err( SdbError::NetworkTimeout )
    }
    else if base.is_connect() {
        Err( SdbError::ConnectionRefused { url } )
    }
    else {
        Err( SdbError::HttpNetowrkError( base ) )
    }
}