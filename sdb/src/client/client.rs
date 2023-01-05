use std::sync::{Arc, RwLock};

use crate::{
    protocols::SdbProtocol,
    client::ClientBuilder,
    reply::TransactionReply,
    transaction::TransactionBuilder, 
    server_info::ServerInfo,
    error::SdbResult,
};


#[derive(Clone, Debug)]
pub struct SurrealClient {
    server: ServerInfo,
    pub(crate) interface: Arc<RwLock<dyn SdbProtocol>>,
}

impl SurrealClient {
    pub fn new(url: &str) -> ClientBuilder {
        ClientBuilder::new(url)
    }

    pub fn server(&self) -> &ServerInfo {
        &self.server
    }

    pub fn transaction(&self) -> TransactionBuilder {
        TransactionBuilder::new(self)
    }

    pub(crate) fn build(server: ServerInfo, interface: impl SdbProtocol + 'static) -> Self {
        Self {
            server,
            interface: Arc::new( RwLock::new( interface ) ),
        }
    }

    pub async fn query( &mut self, trans: TransactionBuilder ) -> SdbResult<TransactionReply> {
        let (queries, sqls) = trans.queries();
        let full_sql = sqls.join(";\n");

        #[cfg(feature = "log")]
        log::debug!("SDB Transaction >\n{full_sql}");

        let mut interface = self.interface.write()?;
        let replies = interface.query( &self.server, full_sql).await?;
        
        Ok( TransactionReply::new(queries, replies) )
    }
}

unsafe impl Sync for SurrealClient { }
unsafe impl Send for SurrealClient { }





