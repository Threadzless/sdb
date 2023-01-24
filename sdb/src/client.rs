use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard};
use serde_json::{Map, Value};

use crate::prelude::*;
 
mod builder;
pub mod interface;

pub use builder::*;
pub use interface::*;

/// The URL to access the demo database. See [`demo()`](fn@SurrealClient::demo)
const DEMO_URL: &str = "ws://demo_user:demo_pass@127.0.0.1:8000/example/demo";

#[derive(Clone)]
pub struct SurrealClient {
    inner: Arc<ClientInner>,
}

impl SurrealClient {
    pub fn open(url: &str) -> ClientBuilder {
        ClientBuilder::new(url)
    }

    /// Create a client for accessing the demo database, and polls it to ensure its running.
    /// 
    /// The demo database is launched by running [`launch-demo-db.sh`](launch_demo-db.sh)
    ///
    /// This method is to make tests and demos more convienient, and shouldn't be used in outside of those cases.
    // #[cfg(any(test, doctest, feature = "extras"))]
    pub fn demo() -> Self {
        Self::open(DEMO_URL).build().unwrap()
    }

    /// What server am I connecting to?
    pub fn server(&self) -> RwLockReadGuard<ServerInfo> {
        self.inner.server()
    }

    pub(crate) fn build(
        server: ServerInfo,
        interface: impl SurrealInterface + 'static,
    ) -> SdbResult<Self> {
 
        let inner = ClientInner {
            socket: Box::new(Mutex::new( interface )),
            server: RwLock::new( server )
        };

        Ok(Self {
            inner: Arc::new( inner )
        })
    }

    /// Create a new [`TransactionBuilder`], for performing queries
    pub fn transaction(&self) -> TransactionBuilder {
        TransactionBuilder::new(self)
    }

    /// Execute a Transaction and return the server's reply
    pub async fn query(&mut self, trans: TransactionBuilder) -> SdbResult<TransactionReply> {
        let (queries, sqls) = trans.queries();
        let full_sql = sqls.join(";\n\t");

        #[cfg(feature = "log")]
        log::info!("Sending Query: \n\t{}\n", full_sql);
        println!("Sending Query: \n\t{}\n", full_sql);

        let request = SurrealRequest::query(full_sql);
        match self.run_request(request).await? {
            SurrealResponse::Error { error, .. } => {
                #[cfg(feature = "log")]
                log::error!("SurrealDB response: {:?}", error);

                panic!("Surreal Responded with an error\n{error:#?}\n");
            }
            SurrealResponse::Result { result, .. } => match result {
                Some(res) => Ok(TransactionReply::new(queries, res)),
                None => {
                    println!("~ ~\n{result:?}\n");
                    panic!()
                }
            },
        }
    }

    pub(crate) async fn run_request(&mut self, request: SurrealRequest) -> SdbResult<SurrealResponse> {
        let req_id = request.id;
        let mut socket = self.inner.socket.lock().unwrap();
        let response = socket.execute(&self.inner.server(), request).await?;
        if !response.check_id(req_id) {
            unreachable!(
                "Packets recieved out of order. {:?} {req_id:?}. Plz report to github",
                response.id()
            )
        }
        Ok( response )
    }

    /// Verifies that that current connection settings are accepted by the server.
    /// 
    /// If you don't call this method, the first async method call on this client will
    /// perform the verification automatically, so using this is optional, but recomended
    /// 
    /// TODO: parse better
    pub async fn handshake<'a>(&'a mut self) -> SdbResult<TransactionReply> {
        // let mut lock = self.inner.socket.lock().unwrap();
        // let info = &self.server();
        // lock.ensure_connected(&info).await?;
        self.transaction()
            .push("INFO FOR DB")
            .run()
            .await
    }

    /// Change which database queries will act on and verifies that
    /// the server will accept this
    pub async fn change_db(&mut self, new_db: &str) -> SdbResult<()>{
        let ns = &self.server().namespace.clone();
        self.change_ns(&ns, new_db).await
    }

    /// Change which namespace and database queries will act on and verifies that
    /// the server will accept this
    pub async fn change_ns(&mut self, new_ns: &str, new_db: &str) -> SdbResult<()> {
        let req = SurrealRequest::use_ns_db( &self.server().namespace, new_db );
        let response = self.run_request(req).await?;
        match response {
            SurrealResponse::Error { .. } => todo!(),
            SurrealResponse::Result { .. } => {

            },
        }

        let mut new_server = self.server().clone();
        new_server.namespace = new_ns.to_string();
        new_server.database = new_db.to_string();
        self.inner.change_server(new_server);
        Ok( () )
    }

    /// Change which authentication credentials and verifies the server accepts them
    pub async fn change_auth(&mut self, new_auth: Option<Credentials>) -> SdbResult<TransactionReply> {
        let mut new_server = self.server().clone();
        new_server.auth = new_auth;
        self.inner.change_server(new_server);
        self.handshake().await
    }

    // Attempts to retrieve a single record from the database by its id.
    pub async fn fetch<R: SurrealRecord>(&mut self, _record_id: RecordId) -> SdbResult<Option<R>> {
        todo!()
    }

    /// Update a single record, using the corresponding record struct.
    pub async fn update<R>(&mut self, mode: UpdateMode, record: &R) -> SdbResult<TransactionReply>
    where
        R: SurrealRecord
    {
        let fields = record.record_fields();
        let contents = obj_to_contents( &fields );
        let rid = record.id();

        let sql = match mode {
            UpdateMode::Content => format!("UPDATE {rid} CONTENT {contents} RETURN NONE"),
            UpdateMode::Patch => format!("UPDATE {rid} PATCH {contents} RETURN NONE"),
            UpdateMode::Merge => format!("UPDATE {rid} MERGE {contents} RETURN NONE"),
        };

        self.transaction()
            .push(&sql)
            .run()
            .await
    }

    /// Create a new record. When creating the record in rust, set the `id` field
    /// to `RecordId::placeholder()` to have the SurrealDb server generate the id for
    /// you.
    pub async fn create<R>(&mut self, record: R) -> SdbResult<R>
    where
        R: SurrealRecord
    {
        let fields = record.record_fields();
        let contents = obj_to_contents( &fields );
        let rid = record.id();
        let create = match rid.is_placeholder() {
            true => rid.table(),
            false => rid.to_string(),
        };

        let mut reply = self.transaction()
            .push(&format!("CREATE {create} CONTENT {contents} RETURN *"))
            .run()
            .await?;

        reply.next_one::<R>()
    }

    // TODO: maybe this needs to be a macro?
    // pub async fn insert_into(&mut self, table: &str, field_names: (&str))
}

//
//
//

unsafe impl Sync for ClientInner {}
unsafe impl Send for ClientInner {}
pub(crate) struct ClientInner {
    socket: Box<Mutex<dyn SurrealInterface>>,
    server: RwLock<ServerInfo>,
}

impl ClientInner {
    pub(crate) fn server(&self) -> RwLockReadGuard<ServerInfo> {
        self.server.read().unwrap()
    }

    pub(crate) fn change_server(&self, new_server: ServerInfo) {
        let Ok( mut old_server ) = self.server.write() else { unreachable!() };
        new_server.clone_into( &mut old_server );
    }
}


/// How records be altered by an update clause
pub enum UpdateMode {
    /// Replace the current record with these values
    Content,
    /// Change and append fields to an existing record, shallowly
    Patch,
    /// Change and append fields to an existing record, deaply
    Merge,
}

fn obj_to_contents( fields: &Map<String, Value> ) -> String {
    let val = fields.iter()
        .filter_map(|(key, val)| {
            if key.eq("id") { return None };
            let s = serde_json::to_string(val).unwrap();
            Some(format!("{key}: {s}"))
        })
        .collect::<Vec<String>>()
        .join(", ");

    format!("{{{}}}", val)
}