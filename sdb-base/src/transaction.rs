use std::thread::JoinHandle;

use crate::{
    client::SurrealClient, error::SdbResult, record::ToSurrealQL, reply::TransactionReply,
};

pub struct TransQuery {
    pub(crate) sql: String,
    pub(crate) skip: bool,
}

pub struct TransactionBuilder {
    client: SurrealClient,
    queries: Vec<TransQuery>,
}

impl TransactionBuilder {
    pub fn new(client: &SurrealClient) -> Self {
        Self {
            client: client.clone(),
            queries: Vec::new(),
        }
    }

    pub fn push(mut self, skip: bool, sql: impl ToString) -> Self {
        self.queries.push(TransQuery {
            sql: sql.to_string(),
            skip,
        });
        self
    }

    pub fn push_var<T: ToSurrealQL>(mut self, var_name: &str, value: T) -> Self {
        self.queries.push(TransQuery {
            sql: format!("LET ${var_name} = {}", value.to_sql()),
            skip: true,
        });
        self
    }

    pub fn query_to_var(mut self, var_name: &str, query: impl ToString) -> Self {
        self.queries.push(TransQuery {
            sql: format!("LET ${var_name} = ({})", query.to_string()),
            skip: true,
        });
        self
    }

    pub(crate) fn queries(self) -> (Vec<TransQuery>, Vec<String>) {
        let sqls = self
            .queries
            .iter()
            .map(|q| q.sql.clone())
            .collect::<Vec<String>>();

        (self.queries, sqls)
    }

    /// Executes the queries and returns the results
    pub async fn run(self) -> SdbResult<TransactionReply> {
        let mut client = self.client.clone();
        client.query(self).await
    }

    // pub fn run_blocking(self) -> JoinHandle<SdbResult<TransactionReply>> {
    //     let mut client = self.client.clone();
    //     client.query(self).

    // }
}