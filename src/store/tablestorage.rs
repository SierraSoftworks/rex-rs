use super::*;
use crate::models::*;
use crate::api::APIError;
use std::{fmt::Debug, sync::{Arc}};
use rand::seq::IteratorRandom;
use actix::prelude::*;
use azure_sdk_storage_table::{CloudTable, Continuation, TableClient, TableEntity};
use serde::Serialize;
use serde::de::DeserializeOwned;

pub struct TableStorage {
    started_at: chrono::DateTime<chrono::Utc>,

    ideas: Arc<CloudTable>,
    role_assignments: Arc<CloudTable>,
    collections: Arc<CloudTable>,
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
        let collections_table = CloudTable::new(client, "collections");

        Self {
            started_at: chrono::Utc::now(),

            ideas: Arc::new(ideas_table),
            collections: Arc::new(collections_table),
            role_assignments: Arc::new(role_assignments_table),
        }
    }

    async fn get_single<ST, T>(table: Arc<CloudTable>, partition_key: u128, row_key: u128, not_found_err: APIError) -> Result<T, APIError>
    where
        ST: DeserializeOwned + Clone,
        T: From<TableEntity<ST>> {
        let result = table.get::<ST>(
            &format!("{:0>32x}", partition_key), 
            &format!("{:0>32x}", row_key),
            None
        ).await
            .map_err(|err| {
                error!("Failed to query Azure Table Storage: {:?}", err);
                APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later.")
            })?;

        result
            .ok_or(not_found_err)
            .map(|r| r.into())
    }

    async fn get_all<ST, T, P>(table: Arc<CloudTable>, query: String, filter: P) -> Result<Vec<T>, APIError>
    where
        ST: Serialize + DeserializeOwned + Clone,
        P: Fn(&TableEntity<ST>) -> bool,
        T: From<TableEntity<ST>>
    {
        let mut continuation = Continuation::start();

        let mut entries: Vec<TableEntity<ST>> = vec![];
        let safe_query = TableStorage::escape_query(query);

        while let Some(mut results) = table.execute_query::<ST>(if safe_query.is_empty() { None } else { Some(safe_query.as_str()) }, &mut continuation).await
            .map_err(|err| {
                error!("Failed to query Azure Table Storage: {:?}", err);
                APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later.")
            })? {

            entries.append(&mut results);
        }

        Ok(entries.iter().filter(|&e| filter(e)).map(|e| e.clone().into()).collect())
    }

    async fn get_random<ST, T, P>(table: Arc<CloudTable>, query: String, filter: P, not_found_err: APIError) -> Result<T, APIError>
    where
        ST: Serialize + DeserializeOwned + Clone,
        P: Fn(&TableEntity<ST>) -> bool,
        T: From<TableEntity<ST>>
    {
        let mut continuation = Continuation::start();

        let mut entries: Vec<TableEntity<ST>> = vec![];
        let safe_query = TableStorage::escape_query(query);

        while let Some(mut results) = table.execute_query::<ST>(if safe_query.is_empty() { None } else { Some(safe_query.as_str()) }, &mut continuation).await
            .map_err(|err| {
                error!("Failed to query Azure Table Storage: {:?}", err);
                APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later.")
            })? {

            entries.append(&mut results);
        }

        entries.iter().filter(|&e| filter(e)).choose(&mut rand::thread_rng()).map(|e| e.clone().into()).ok_or(not_found_err)
    }

    async fn store_single<ST, T>(table: Arc<CloudTable>, item: TableEntity<ST>) -> Result<T, APIError> 
    where
        ST: Serialize + DeserializeOwned + Clone + Debug,
        T: From<TableEntity<ST>> {
        let result = table.insert_or_update_entity(item).await
                .map_err(|err| {
                    error!("Failed to query Azure Table Storage: {:?}", err);
                    APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later.")
                })?;

        Ok(result.into())
    }

    async fn remove_single(table: Arc<CloudTable>, partition_key: u128, row_key: u128) -> Result<(), APIError> {
        table.delete(
            &format!("{:0>32x}", partition_key), 
            &format!("{:0>32x}", row_key),
            None).await
                .map_err(|err| {
                    error!("Failed to query Azure Table Storage: {:?}", err);
                    APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later.")
                })?;

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

    fn escape_query(query: String) -> String {
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
            collection: u128::from_str_radix(&entity.partition_key, 16).unwrap_or_default(),
            name: entity.payload.name.clone(),
            tags: hashset!([entity.payload.tags.split(",")]),
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
            id: u128::from_str_radix(&entity.row_key, 16).unwrap_or_default(),
            principal_id: u128::from_str_radix(&entity.partition_key, 16).unwrap_or_default(),
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
            principal_id: u128::from_str_radix(&entity.row_key, 16).unwrap_or_default(),
            role: entity.payload.role.as_str().into(),
        }
    }
}

macro_rules! actor_handler {
    ($msg:ty => $res:ty: handler = $handler:item) => {
        impl Handler<$msg> for TableStorage {
            type Result = ResponseActFuture<Self, Result<$res, APIError>>;
            
            $handler
        }
    };

    ($msg:ty|$src:ident => $res:ty: get_single from $table:ident ( $st:ty ) where pk=$pk:expr, rk=$rk:expr; not found = $err:expr) => {
        actor_handler!($msg => $res: handler = fn handle(&mut self, $src: $msg, _: &mut Self::Context) -> Self::Result {
            let table = self.$table.clone();
            let work = TableStorage::get_single::<$st, $res>(
                table,
                $pk,
                $rk,
                APIError::new(404, "Not Found", $err));
            Box::new(fut::wrap_future(work))
        });
    };

    ($msg:ty|$src:ident => $res:ty: get_all from $table:ident ( $st:ty ) where query = $query:expr, context = [$($ctx:tt)*], filter = $fid:ident -> $filter:expr) => {
        actor_handler!($msg => Vec<$res>: handler = fn handle(&mut self, $src: $msg, _: &mut Self::Context) -> Self::Result {
            let table = self.$table.clone();
            let query = $query;

            $($ctx)*

            let work = TableStorage::get_all::<$st, $res, _>(
                table,
                query,
                move |$fid| $filter
            );

            Box::new(fut::wrap_future(work))
        });
    };

    ($msg:ty|$src:ident => $res:ty: get_random from $table:ident ( $st:ty ) where query = $query:expr, context = [$($ctx:tt)*], filter = $fid:ident -> $filter:expr; not found = $err:expr) => {
        actor_handler!($msg => $res: handler = fn handle(&mut self, $src: $msg, _: &mut Self::Context) -> Self::Result {
            let table = self.$table.clone();
            let query = $query;

            $($ctx)*

            let work = TableStorage::get_random::<$st, $res, _>(
                table,
                query,
                move |$fid| $filter,
                APIError::new(404, "Not Found", $err)
            );

            Box::new(fut::wrap_future(work))
        });
    };

    ($msg:ty|$src:ident: remove_single from $table:ident where pk=$pk:expr, rk=$rk:expr) => {
        actor_handler!($msg => (): handler = fn handle(&mut self, $src: $msg, _: &mut Self::Context) -> Self::Result {
            let table = self.$table.clone();
            let work = TableStorage::remove_single(
                table,
                $pk,
                $rk);
            Box::new(fut::wrap_future(work))
        });
    };
    
    ($msg:ty|$src:ident => $res:ty: store_single in $table:ident ( $st:ty ) $item:expr) => {
        actor_handler!($msg => $res: handler = fn handle(&mut self, $src: $msg, _: &mut Self::Context) -> Self::Result {
            let table = self.$table.clone();
            let item = $item;
            let work = TableStorage::store_single::<$st, $res>(
                table,
                item
            );

            Box::new(fut::wrap_future(work))
        });
    };
}

impl Actor for TableStorage {
    type Context = Context<Self>;
}

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
        tags: msg.tags.iter().fold("".to_string(), |j, i| j + i.as_str()),
        completed: false,
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