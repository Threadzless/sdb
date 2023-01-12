use serde::{Deserialize, Serialize};

use crate::{client::SurrealClient, error::*, reply::TransactionReply};

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

    /// Insert a new query into the transaction, which will produce a result
    /// when the transaction is run
    ///
    /// ## Example
    /// ```
    /// # use sdb_base::prelude::*;
    /// # use serde::{Serialize, Deserialize};
    /// #
    /// # tokio_test::block_on( async {
    /// # let client = SurrealClient::demo();
    /// #
    /// let mut reply = client.transaction()
    ///     .push("SELECT * FROM books LIMIT 5")
    ///     .run()
    ///     .await.unwrap();
    ///     
    /// let five_books: Vec<BookSchema> = reply.next_vec().unwrap();
    /// # });
    /// #
    /// # #[derive(Clone, Deserialize)]
    /// # pub struct BookSchema {
    /// #     pub id: RecordId,
    /// #     pub title: String,
    /// #     pub word_count: Option<usize>,
    /// # }
    /// ```
    pub fn push(mut self, sql: &str) -> Self {
        self.queries.push(TransQuery {
            sql: sql.to_string(),
            skip: false,
        });
        self
    }

    /// Inserts a value into the transaction. Queries pushed in after this can use
    /// the variable. Inserting will sanitise the value as needed for its type.
    ///
    /// Accepts anything which implements [`serde::Serialize`], which is all
    /// primitives, everything in [`std::collections`], and a lot more.
    ///
    /// ## Example
    /// ```
    /// # use sdb_base::prelude::*;
    /// # use serde::{Serialize, Deserialize};
    /// #
    /// # tokio_test::block_on( async {
    /// # let client = SurrealClient::demo();
    /// #
    /// let mut reply = client.transaction()
    ///     .push_var("search_term", "Spacetime")
    ///     .push("SELECT * FROM books WHERE title ~ $search_term")
    ///     .run()
    ///     .await.unwrap();
    ///     
    /// let books: Vec<BookSchema> = reply.next_vec().unwrap();
    /// # });
    /// #
    /// # #[derive(Clone, Deserialize)]
    /// # pub struct BookSchema {
    /// #     pub id: RecordId,
    /// #     pub title: String,
    /// #     pub word_count: Option<usize>,
    /// # }
    /// ```
    pub fn push_var<T: Serialize>(mut self, var_name: &str, value: T) -> Self {
        match serde_json::to_string(&value) {
            Err(_e) => panic!("Cannot serialize value into variable `{var_name}`"),
            Ok(val_string) => {
                self.queries.push(TransQuery {
                    sql: format!("LET ${var_name} = {val_string}"),
                    skip: true,
                });
                self
            }
        }
    }

    /// Insert a new query into the transaction, which will *NOT* produce a result
    /// when the transaction is run.
    ///
    /// Adding skipped queries into transaction will not alter the contents
    /// of the transaction results, so adding them doesn't require you to
    /// reassess the ordering of you `.next_*` calls.
    ///
    /// ## Example
    /// ```
    /// # use sdb_base::prelude::*;
    /// # use serde::{Serialize, Deserialize};
    /// #
    /// # tokio_test::block_on( async {
    /// # let client = SurrealClient::demo();
    /// #
    /// let mut reply = client.transaction()
    ///     .push_skipped("USE DB demo")
    ///     .push("SELECT * FROM books LIMIT 5")
    ///     .run()
    ///     .await.unwrap();
    ///     
    /// let five_books: Vec<BookSchema> = reply.next_vec().unwrap();
    /// # });
    /// #
    /// # #[derive(Clone, Deserialize)]
    /// # pub struct BookSchema {
    /// #     pub id: RecordId,
    /// #     pub title: String,
    /// #     pub word_count: Option<usize>,
    /// # }
    /// ```
    pub fn push_skipped(mut self, sql: &str) -> Self {
        self.queries.push(TransQuery {
            sql: sql.to_string(),
            skip: true,
        });
        self
    }

    /// Executes a query and stores its results in a transaction variable.
    ///
    /// ```rust
    /// # use sdb_base::prelude::*;
    /// # use sdb_macros::*;
    /// # use sdb_base as sdb;
    /// # use serde::{Serialize, Deserialize};
    /// #
    /// # async fn main_test() {
    /// # let client = SurrealClient::demo();
    /// #
    /// let mut reply = client.transaction()
    ///     .push_var("name_search", "George R. R. Martin")
    ///     .query_to_var("good_books", "SELECT * FROM books WHERE authors.name ~ $name_search")
    ///     .push("SELECT * FROM count(($good_books))")
    ///     .run()
    ///     .await
    ///     .unwrap();
    ///      
    /// let good_books: i32 = reply.next_one().unwrap();
    /// # }
    /// #
    /// # tokio_test::block_on( async {
    /// #     main_test().await
    /// # });
    /// #
    /// # #[derive(Clone, Deserialize, SurrealRecord)]
    /// # pub struct BookSchema {
    /// #     pub id: RecordId,
    /// #     pub title: String,
    /// #     pub word_count: Option<usize>,
    /// # }
    /// ```
    pub fn query_to_var(mut self, var_name: &str, query: &str) -> Self {
        self.queries.push(TransQuery {
            sql: format!("LET ${var_name} = ({query})"),
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

    /// Executes the transaction and returns the results
    pub async fn run(self) -> SdbResult<TransactionReply> {
        let mut client = self.client.clone();
        client.query(self).await
    }

    /// Executes the transaction and then parses and returns a list of results from
    /// the first non-skipped query
    pub async fn run_parse_vec<T>(self) -> SdbResult<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut reply = self.run().await?;
        reply.next_vec::<T>()
    }

    /// Executes the transaction and then parses and returns a single result from
    /// the first non-skipped query
    pub async fn run_parse_opt<T>(self) -> SdbResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut reply = self.run().await?;
        reply.next_opt::<T>()
    }

    /// Executes the transaction and then parses and returns a non-optional single
    /// result from the first non-skipped query
    pub async fn run_parse_one<T>(self) -> SdbResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut reply = self.run().await?;
        reply.next_one::<T>()
    }
}
