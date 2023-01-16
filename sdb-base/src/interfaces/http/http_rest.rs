use reqwest::{Client, ClientBuilder, Error as ReqError, RequestBuilder, Url};
use serde_json::Value;

use crate::{
    client::interface::*,
    error::{SdbError, SdbResult},
    reply::QueryReply,
    server_info::ServerInfo,
};

#[derive(Debug)]
pub struct HttpSurrealInterface {
    client: Client,
}

impl SurrealInterfaceBuilder for HttpSurrealInterface {
    fn new(_info: &ServerInfo) -> SdbResult<Self> {
        let client = ClientBuilder::new().build().unwrap();

        Ok(Self { client })
    }
}

impl HttpSurrealInterface {
    fn request(&self, info: &ServerInfo) -> Result<RequestBuilder, SdbError> {
        let url = Url::parse(&info.full_url()).expect("A valid hostname for the database");

        let mut req = self.client.post(url);

        for (k, v) in info.headers() {
            req = req.header(k, v);
        }

        Ok(req)
    }

    async fn execute_query( &self, info: &ServerInfo, sql: impl ToString ) -> SdbResult<Vec<QueryReply>> {
        let req = self.request(info)?.body( sql.to_string() );
        let res = req.send().await.map_err(|e| convert_err(e, info))?;
        let txt = res.text().await.map_err(|e| convert_err(e, info))?;
        match serde_json::from_str::<Vec<QueryReply>>(&txt) {
            Err(_err) => unreachable!("Response Parse Failure"),
            Ok(replies) => Ok(replies),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl SurrealInterface for HttpSurrealInterface {
    async fn execute(
        &mut self,
        info: &ServerInfo,
        request: SurrealRequest,
    ) -> SdbResult<SurrealResponse> {

        match request.method {
            RequestMethod::Ping => unreachable!(),
            RequestMethod::Info => todo!(),
            RequestMethod::Select => todo!(),
            RequestMethod::Create => todo!(),
            RequestMethod::Update => todo!(),
            RequestMethod::Merge => todo!(),
            RequestMethod::Patch => todo!(),
            RequestMethod::Delete => todo!(),
            RequestMethod::Format => todo!(),
            RequestMethod::Version => todo!(),
            RequestMethod::Query => {
                let Some( Value::String( sql ) ) = request.params.get(0) else { unreachable!() };
                let replies = self.execute_query(info, sql).await?;
                Ok( SurrealResponse::Result {
                    id: request.id,
                    result: Some(replies),
                })
            },

            RequestMethod::Use |
            RequestMethod::Let | 
            RequestMethod::Unset => todo!("Don't really make sense to use here"),

            RequestMethod::Kill |
            RequestMethod::Live => todo!("Unimplementable for this interface"),

            RequestMethod::Signup |
            RequestMethod::Signin |
            RequestMethod::Invalidate |
            RequestMethod::Authenticate => todo!("Alter server-info or do nothing"),
        }
    }
}

fn convert_err(base: ReqError, info: &ServerInfo) -> SdbError {
    let url = info.full_url();

    if base.is_timeout() {
        SdbError::NetworkTimeout
    } else if base.is_connect() {
        SdbError::ConnectionRefused { url }
    } else {
        SdbError::HttpNetowrkError(base)
    }
}
