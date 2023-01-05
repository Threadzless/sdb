use async_trait::async_trait;

use reqwest::{Client, ClientBuilder, RequestBuilder, Url};

use crate::{
    server_info::ServerInfo,
    protocols::SdbProtocol,
    reply::QueryReply,
    // Credentials,
    error::SdbError,
};


#[derive(Debug)]
pub struct HttpProtocol {
    client: Client,
}

impl HttpProtocol {
    pub fn new(_info: &ServerInfo) -> Self {
        
        let client = ClientBuilder::new()
            .build()
            .unwrap();

        Self {
            client,
        }
    }

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
impl SdbProtocol for HttpProtocol {
    async 
    fn connect_if_not(&mut self, _info: &ServerInfo) -> Result<(), SdbError> {
        // let req = self.request("INFO FOR DB")?;
        // match req.send().await {
        //     Ok(_db) => Ok( vec![]),
        //     Err( e ) => {
        //         println!("{:?}", self.info );
        //         todo!("CORS error?\n\n{:?}", e )
        //     },
        // }
        Ok( () )
    }

    async 
    fn query(&mut self, info: &ServerInfo, sql: String) -> Result<Vec<QueryReply>, SdbError> {
        let req = self.request(info, sql)?;

        let res = match req.send().await {
            Ok(res) => res,
            Err(err) => panic!("Netork Error: {err:?}"),
        };

        let text = res.text().await.unwrap();
        Ok( serde_json::from_str::<Vec<QueryReply>>( &text ).unwrap() )
    }
}