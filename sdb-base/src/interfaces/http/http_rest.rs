use reqwest::{Client, ClientBuilder, RequestBuilder, Url, Error as ReqError};
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

    async fn send(&mut self, info: &ServerInfo, request: SurrealRequest) -> SdbResult<SurrealResponse> {
        let Some( Value::String( sql ) ) = request.params.get(0) else { unreachable!() };
        let req = self.request(info, sql)?;
        let res = req.send().await.map_err( |e| convert_err(e, info) )?;
        let txt = res.text().await;
        let txt = txt.map_err( |e| convert_err(e, info) )?;
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

fn convert_err(
    base: ReqError, 
    info: &ServerInfo
) -> SdbError {

    let url = info.full_url();

    if base.is_timeout() {
        SdbError::NetworkTimeout
    }
    else if base.is_connect() {
        SdbError::ConnectionRefused { url }
    }
    else {
        SdbError::HttpNetowrkError( base )
    }
}