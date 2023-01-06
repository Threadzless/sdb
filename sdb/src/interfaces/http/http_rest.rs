use async_trait::async_trait;

use reqwest::{Client, ClientBuilder, RequestBuilder, Url};
use serde_json::Value;
// use serde_json::Value;

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

// #[cfg(not( target_family = "wasm"))]
#[async_trait(?Send)]
impl SurrealInterface for HttpSurrealInterface {
    // fn send(&mut self, info: &ServerInfo, sql: String) -> Result<Vec<QueryReply>, SdbError> {
    async fn send(&mut self, info: &ServerInfo, request: SurrealRequest) -> SdbResult<SurrealResponse> {

        let Some( Value::String( sql ) ) = request.params.get(0) else { panic!() };

        let req = self.request(info, sql)?;

        let res = req.send().await?;

        let text = res.text().await.unwrap();
        match serde_json::from_str::<Vec<QueryReply>>( &text ) {
            Err( _err ) => panic!("Failed to parse"),
            Ok( replies ) => Ok(
                SurrealResponse::Result {
                    id: request.id, 
                    result: Some( replies) 
                }
            )
        }

        // todo!()
    }
}