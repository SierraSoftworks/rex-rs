use crate::{models::*, trace_handler};
use crate::api::APIError;
use std::{convert::TryInto, fmt::Debug, pin::Pin, sync::Arc};
use rand::seq::IteratorRandom;
use actix::prelude::*;
use azure_sdk_storage_core::key_client::KeyClient;
use azure_sdk_storage_table::{CloudTable, Continuation, TableClient, TableEntity};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tracing::Instrument;

type TableReference = Arc<CloudTable<KeyClient>>;

pub struct TableStorage {
    started_at: chrono::DateTime<chrono::Utc>,

    ideas: TableReference,
    role_assignments: TableReference,
    collections: TableReference,
    users: TableReference,
}

const URI_CHARACTERS: &percent_encoding::AsciiSet = &percent_encoding::CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'<')
    .add(b'>')
    .add(b'%')
    .add(b'#')
    .add(b'&');

impl TableStorage {
    pub fn new() -> Self {
        let connection_string = std::env::var("TABLE_STORAGE_CONNECTION_STRING").expect("Set the TABLE_STORAGE_CONNECTION_STRING environment variable before starting the server.");

        let client = TableClient::from_connection_string(&connection_string).expect("a valid connection string");
        let ideas_table = CloudTable::new(client.clone(), "ideas");
        let role_assignments_table = CloudTable::new(client.clone(), "roleassignments");
        let collections_table = CloudTable::new(client.clone(), "collections");
        let users_table = CloudTable::new(client, "users");

        Self {
            started_at: chrono::Utc::now(),

            ideas: TableReference::new(ideas_table),
            collections: TableReference::new(collections_table),
            role_assignments: TableReference::new(role_assignments_table),
            users: TableReference::new(users_table),
        }
    }

    #[instrument(err, skip(table, not_found_err), fields(otel.kind = "client", db.system = "azure_table_storage", db.operation = "GET"))]
    async fn get_single<ST, T>(table: TableReference, type_name: &str, partition_key: u128, row_key: u128, not_found_err: APIError) -> Result<T, APIError>
    where
        ST: DeserializeOwned + Clone,
        T: From<TableEntity<ST>> {
        let result = table.get::<ST>(
            &format!("{:0>32x}", partition_key), 
            &format!("{:0>32x}", row_key),
            None
        ).await?;

        result
            .ok_or(not_found_err)
            .map(|r| r.into())
    }

    #[instrument(err, skip(table, filter), fields(otel.kind = "client", db.system = "azure_table_storage", db.operation = "LIST", db.statement = %query))]
    async fn get_all<ST, T, P>(table: TableReference, type_name: &str, query: String, filter: P) -> Result<Vec<T>, APIError>
    where
        ST: Serialize + DeserializeOwned + Clone,
        P: Fn(&TableEntity<ST>) -> bool,
        T: From<TableEntity<ST>>
    {
        let mut continuation = Continuation::start();

        let mut entries: Vec<TableEntity<ST>> = vec![];
        let safe_query = TableStorage::escape_query(&query);

        while let Some(mut results) = table.execute_query::<ST>(
            if safe_query.is_empty() { None } else { Some(safe_query.as_str()) }
            , &mut continuation
        ).instrument(
            info_span!("Fetching page of results from Table Storage", "otel.kind" = "client", "db.system" = "azure_table_storage", "db.operation" = "LIST", db.statement = %query)
        ).await? {
            entries.append(&mut results);
        }

        Ok(entries.iter().filter(|&e| filter(e)).map(|e| e.clone().into()).collect())
    }

    #[instrument(err, skip(table, filter, not_found_err), fields(otel.kind = "client", db.system = "azure_table_storage", db.operation = "LIST", db.statement = %query))]
    async fn get_random<ST, T, P>(table: TableReference, type_name: &str, query: String, filter: P, not_found_err: APIError) -> Result<T, APIError>
    where
        ST: Serialize + DeserializeOwned + Clone,
        P: Fn(&TableEntity<ST>) -> bool,
        T: From<TableEntity<ST>>
    {
        let mut continuation = Continuation::start();

        let mut entries: Vec<TableEntity<ST>> = vec![];
        let safe_query = TableStorage::escape_query(&query);

        while let Some(mut results) = table.execute_query::<ST>(
            if safe_query.is_empty() { None } else { Some(safe_query.as_str()) },
            &mut continuation
        ).instrument(
            info_span!("Fetching page of results from Table Storage", "otel.kind" = "client", "db.system" = "azure_table_storage", "db.operation" = "LIST", db.statement = %query)
        ).await? {
            entries.append(&mut results);
        }

        entries.iter().filter(|&e| filter(e)).choose(&mut rand::thread_rng()).map(|e| e.clone().into()).ok_or(not_found_err)
    }

    #[instrument(err, skip(table, item), fields(otel.kind = "client", db.system = "azure_table_storage", db.operation = "PUT"))]
    async fn store_single<ST, T>(table: TableReference, type_name: &str, item: TableEntity<ST>) -> Result<T, APIError> 
    where
        ST: Serialize + DeserializeOwned + Clone + Debug,
        T: From<TableEntity<ST>> {
        
        let result = table.insert_or_update_entity(item).await?;

        Ok(result.into())
    }

    #[instrument(err, skip(table), fields(otel.kind = "client", db.system = "azure_table_storage", db.operation = "DELETE"))]
    async fn remove_single(table: TableReference, type_name: &str, partition_key: u128, row_key: u128) -> Result<(), APIError> {
        table.delete(
            &format!("{:0>32x}", partition_key), 
            &format!("{:0>32x}", row_key),
            None).await?;
        
        Ok(())
    }

    fn build_idea_filter_query(partition_key: u128, is_completed: Option<bool>, tag: Option<String>) -> String {
        let mut query = format!("$filter=PartitionKey eq '{:0>32x}'", partition_key);
        match is_completed {
            Some(completed) => {
                query = query + format!(" and Completed eq {}", completed).as_str()
            },
            None => {}
        }
        
        match tag {
            Some(tag) => {
                query = query + format!(" and contains(Tags, '{}')", tag.replace("'", "''").replace("%", "%25")).as_str()
            },
            None => {}
        }

        query
    }

    fn escape_query(query: &str) -> String {
        percent_encoding::percent_encode(query.as_bytes(), URI_CHARACTERS).to_string()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TableStorageIdea {
    #[serde(rename="Name")]
    pub name: String,
    #[serde(rename="Description")]
    pub description: String,
    #[serde(rename="Tags")]
    pub tags: String,
    #[serde(rename="Completed")]
    pub completed: bool,
}

impl From<TableEntity<TableStorageIdea>> for Idea {
    fn from(entity: TableEntity<TableStorageIdea>) -> Self {
        Self {
            id: u128::from_str_radix(&entity.row_key, 16).unwrap_or_default(),
            collection_id: u128::from_str_radix(&entity.partition_key, 16).unwrap_or_default(),
            name: entity.payload.name.clone(),
            tags: hashset!([entity.payload.tags.split(",").filter(|t| !t.is_empty())]),
            description: entity.payload.description.clone(),
            completed: entity.payload.completed
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TableStorageCollection {
    #[serde(rename="Name")]
    pub name: String,
}

impl From<TableEntity<TableStorageCollection>> for Collection {
    fn from(entity: TableEntity<TableStorageCollection>) -> Self {
        Self {
            collection_id: u128::from_str_radix(&entity.row_key, 16).unwrap_or_default(),
            user_id: u128::from_str_radix(&entity.partition_key, 16).unwrap_or_default(),
            name: entity.payload.name.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TableStorageRoleAssignment {
    #[serde(rename="Role")]
    pub role: String,
}

impl From<TableEntity<TableStorageRoleAssignment>> for RoleAssignment {
    fn from(entity: TableEntity<TableStorageRoleAssignment>) -> Self {
        Self {
            collection_id: u128::from_str_radix(&entity.partition_key, 16).unwrap_or_default(),
            user_id: u128::from_str_radix(&entity.row_key, 16).unwrap_or_default(),
            role: entity.payload.role.as_str().into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TableStorageUser {
    #[serde(rename="PrincipalId")]
    pub principal_id: String,

    #[serde(rename="FirstName")]
    pub first_name: String,
}

impl From<TableEntity<TableStorageUser>> for User {
    fn from(entity: TableEntity<TableStorageUser>) -> Self {
        Self {
            email_hash: u128::from_str_radix(&entity.partition_key, 16).unwrap_or_default(),
            principal_id: u128::from_str_radix(&entity.payload.principal_id, 16).unwrap_or_default(),
            first_name: entity.payload.first_name.as_str().into(),
        }
    }
}

trait AsyncHandler<M>
where
    M: Message,
{
    type Result;

    // This method is called for every message received by this actor.
    fn handle_internal(&self, msg: M) -> Pin<Box<dyn Future<Output = Self::Result>>>;
}

macro_rules! actor_handler {
    ($msg:ty => $res:ty: handler = $handler:item) => {
        
        impl AsyncHandler<$msg> for TableStorage {
            type Result = Result<$res, APIError>;
            
            $handler
        }

        impl actix::Handler<$msg> for TableStorage {
            type Result = ResponseActFuture<Self, Result<$res, APIError>>;

            fn handle(&mut self, msg: $msg, _ctx: &mut Self::Context) -> Self::Result {
                Box::pin(fut::wrap_future(self.handle_internal(msg)))
            }
        }

        impl actix::Handler<$crate::telemetry::TraceMessage<$msg>> for TableStorage {
            type Result = ResponseActFuture<Self, Result<$res, APIError>>;

            fn handle(&mut self, msg: $crate::telemetry::TraceMessage<$msg>, _ctx: &mut Self::Context) -> Self::Result {
                let work = self.handle_internal(msg.message);

                let instrumentation = async move {
                    work.await
                }.instrument(msg.span);

                Box::pin(fut::wrap_future(instrumentation))
            }
        }
    };

    ($msg:ty|$src:ident => $res:ty: get_single from $table:ident ( $st:ty ) where pk=$pk:expr, rk=$rk:expr; not found = $err:expr) => {
        actor_handler!($msg => $res: handler = fn handle_internal(&self, $src: $msg) -> Pin<Box<dyn Future<Output = Self::Result>>> {
            let table = self.$table.clone();
            let work = TableStorage::get_single::<$st, $res>(
                table,
                "$table",
                $pk,
                $rk,
                APIError::new(404, "Not Found", $err));

            Box::pin(work)
        });
    };

    ($msg:ty|$src:ident => $res:ty: get_all from $table:ident ( $st:ty ) where query = $query:expr, context = [$($ctx:tt)*], filter = $fid:ident -> $filter:expr) => {
        actor_handler!($msg => Vec<$res>: handler = fn handle_internal(&self, $src: $msg) -> Pin<Box<dyn Future<Output = Self::Result>>> {
            let table = self.$table.clone();
            let query = $query;

            $($ctx)*

            let work = TableStorage::get_all::<$st, $res, _>(
                table,
                "$table",
                query,
                move |$fid| $filter
            );

            Box::pin(work)
        });
    };

    ($msg:ty|$src:ident => $res:ty: get_random from $table:ident ( $st:ty ) where query = $query:expr, context = [$($ctx:tt)*], filter = $fid:ident -> $filter:expr; not found = $err:expr) => {
        actor_handler!($msg => $res: handler = fn handle_internal(&self, $src: $msg) -> Pin<Box<dyn Future<Output = Self::Result>>> {
            let table = self.$table.clone();
            let query = $query;

            $($ctx)*

            let work = TableStorage::get_random::<$st, $res, _>(
                table,
                "$table",
                query,
                move |$fid| $filter,
                APIError::new(404, "Not Found", $err)
            );

            Box::pin(work)
        });
    };

    ($msg:ty|$src:ident: remove_single from $table:ident where pk=$pk:expr, rk=$rk:expr) => {
        actor_handler!($msg => (): handler = fn handle_internal(&self, $src: $msg) -> Pin<Box<dyn Future<Output = Self::Result>>> {
            let table = self.$table.clone();
            let work = TableStorage::remove_single(
                table,
                "$table",
                $pk,
                $rk);

            Box::pin(work)
        });
    };
    
    ($msg:ty|$src:ident => $res:ty: store_single in $table:ident ( $st:ty ) $item:expr) => {
        actor_handler!($msg => $res: handler = fn handle_internal(&self, $src: $msg) -> Pin<Box<dyn Future<Output = Self::Result>>> {
            let table = self.$table.clone();
            let item = $item;
            let work = TableStorage::store_single::<$st, $res>(
                table,
                "$table",
                item
            );

            Box::pin(work)
        });
    };
}

impl Actor for TableStorage {
    type Context = Context<Self>;
}

trace_handler!(TableStorage, GetHealth, Result<Health, APIError>);

impl Handler<GetHealth> for TableStorage {
    type Result = Result<Health, APIError>;

    fn handle(&mut self, _: GetHealth, _: &mut Self::Context) -> Self::Result {
        Ok(Health {
            ok: true,
            started_at: self.started_at.clone(),
        })
    }
}

actor_handler!(GetIdea|msg => Idea: get_single from ideas(TableStorageIdea) where pk=msg.collection, rk=msg.id; not found = "The combination of collection and idea ID you provided could not be found. Please check them and try again.");

actor_handler!(GetIdeas|msg => Idea: get_all from ideas(TableStorageIdea) where
    query=TableStorage::build_idea_filter_query(msg.collection, msg.is_completed, msg.tag.clone()),
    context = [
        let tag_str = msg.tag.unwrap_or("".to_string());
    ],
    filter=i -> tag_str.is_empty() || i.payload.tags.split(",").any(|i| i == tag_str.as_str()));

    
actor_handler!(GetRandomIdea|msg => Idea: get_random from ideas(TableStorageIdea) where
    query = TableStorage::build_idea_filter_query(msg.collection, msg.is_completed, msg.tag.clone()),
    context = [
        let tag_str = msg.tag.unwrap_or("".to_string());
    ],
    filter = i -> tag_str.is_empty() || i.payload.tags.split(",").any(|i| i == tag_str.as_str());
    not found = "We could not find any ideas in the collection you provided which matched your query. Please create some and try again.");

actor_handler!(StoreIdea|msg => Idea: store_single in ideas(TableStorageIdea) TableEntity {
    partition_key: format!("{:0>32x}", msg.collection),
    row_key: format!("{:0>32x}", msg.id),
    payload: TableStorageIdea {
        name: msg.name.clone(),
        description: msg.description.clone(),
        tags: msg.tags.iter().fold("".to_string(), |j, i| j + "," + i.as_str()),
        completed: msg.completed,
    },
    etag: None,
    timestamp: None
});

actor_handler!(RemoveIdea|msg: remove_single from ideas where pk=msg.collection, rk=msg.id);

actor_handler!(GetCollection|msg => Collection: get_single from collections(TableStorageCollection) where pk=msg.principal_id, rk=msg.id; not found = "The collection ID you provided could not be found. Please check them and try again.");

actor_handler!(GetCollections|msg => Collection: get_all from collections(TableStorageCollection) where
    query = format!("$filter=PartitionKey eq '{:0>32x}'", msg.principal_id),
    context = [],
    filter = _i -> true);

actor_handler!(StoreCollection|msg => Collection: store_single in collections(TableStorageCollection) TableEntity {
    partition_key: format!("{:0>32x}", msg.principal_id),
    row_key: format!("{:0>32x}", msg.collection_id),
    payload: TableStorageCollection {
        name: msg.name.clone(),
    },
    etag: None,
    timestamp: None
});

actor_handler!(RemoveCollection|msg: remove_single from collections where pk=msg.principal_id, rk=msg.id);

actor_handler!(GetRoleAssignment|msg => RoleAssignment: get_single from role_assignments(TableStorageRoleAssignment) where pk=msg.collection_id, rk=msg.principal_id; not found = "The collection ID you provided could not be found. Please check them and try again.");

actor_handler!(GetRoleAssignments|msg => RoleAssignment: get_all from role_assignments(TableStorageRoleAssignment) where
    query = format!("$filter=PartitionKey eq '{:0>32x}'", msg.collection_id),
    context = [],
    filter = _i -> true);

actor_handler!(StoreRoleAssignment|msg => RoleAssignment: store_single in role_assignments(TableStorageRoleAssignment) TableEntity {
    partition_key: format!("{:0>32x}", msg.collection_id),
    row_key: format!("{:0>32x}", msg.principal_id),
    payload: TableStorageRoleAssignment {
        role: msg.role.into()
    },
    etag: None,
    timestamp: None
}); 

actor_handler!(RemoveRoleAssignment|msg: remove_single from role_assignments where pk=msg.collection_id, rk=msg.principal_id);

actor_handler!(GetUser|msg => User: get_single from users(TableStorageUser) where pk=msg.email_hash, rk=msg.email_hash; not found = "The user you are looking for could not be found. Please check that you have entered their email address correctly and try again.");

actor_handler!(StoreUser|msg => User: store_single in users(TableStorageUser) TableEntity {
    partition_key: format!("{:0>32x}", msg.email_hash),
    row_key: format!("{:0>32x}", msg.email_hash),
    payload: TableStorageUser {
        principal_id: format!("{:0>32x}", msg.principal_id),
        first_name: msg.first_name.clone(),
    },
    etag: None,
    timestamp: None
});