//! Moves Rex's data out of Azure Table Storage and into SQLite.
//!
//! This runs once, at cutover, and is deliberately not part of the server
//! binary. It writes through the real `SqliteStore`, so it cannot invent a
//! schema of its own, and it is idempotent against a fresh file -- which means
//! the cutover can be rehearsed against production data as many times as it
//! takes.
//!
//! ```text
//! TABLE_STORAGE_CONNECTION_STRING=... rex-import-tables rex.sqlite
//! ```

use std::collections::{HashMap, HashSet};

use azure_data_tables::prelude::*;
use azure_storage::StorageCredentials;
use futures::StreamExt;
use rex_server::{
    db::{SqliteStore, Store},
    models::{Collection, Idea, Role, RoleAssignment, User, format_id, parse_id},
};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TableIdea {
    #[serde(rename = "PartitionKey")]
    collection_id: String,
    #[serde(rename = "RowKey")]
    id: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "Tags")]
    tags: String,
    #[serde(rename = "Completed")]
    completed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TableCollection {
    #[serde(rename = "PartitionKey")]
    principal_id: String,
    #[serde(rename = "RowKey")]
    collection_id: String,
    #[serde(rename = "Name")]
    name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TableRoleAssignment {
    #[serde(rename = "PartitionKey")]
    collection_id: String,
    #[serde(rename = "RowKey")]
    principal_id: String,
    #[serde(rename = "Role")]
    role: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TableUser {
    #[serde(rename = "PartitionKey")]
    email_hash: String,
    #[serde(rename = "PrincipalId")]
    principal_id: String,
    #[serde(rename = "FirstName")]
    first_name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let target = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: rex-import-tables <path-to-new-sqlite-file>");
        std::process::exit(2);
    });

    if std::path::Path::new(&target).exists() {
        // Importing into a database that already holds data would silently
        // merge two generations of it.
        eprintln!("{target} already exists; the importer only writes to a fresh file.");
        std::process::exit(2);
    }

    let connection_string = std::env::var("TABLE_STORAGE_CONNECTION_STRING").map_err(
        |_| "Set TABLE_STORAGE_CONNECTION_STRING to the storage account holding the tables.",
    )?;

    let credentials = azure_storage::ConnectionString::new(&connection_string)?;
    let account = credentials
        .account_name
        .ok_or("The connection string must include the account name.")?;
    let key = credentials
        .account_key
        .ok_or("The connection string must include the account key.")?;

    let tables = TableServiceClient::new(
        account,
        StorageCredentials::access_key(account.to_string(), key.to_string()),
    );

    info!("Reading the source tables.");

    let source_collections: Vec<TableCollection> = read_all(&tables, "collections").await?;
    let source_roles: Vec<TableRoleAssignment> = read_all(&tables, "roleassignments").await?;
    let source_ideas: Vec<TableIdea> = read_all(&tables, "ideas").await?;
    let source_users: Vec<TableUser> = read_all(&tables, "users").await?;

    info!(
        "Read {} collections, {} role assignments, {} ideas, and {} users.",
        source_collections.len(),
        source_roles.len(),
        source_ideas.len(),
        source_users.len()
    );

    // The store applies the real migrations, so the importer cannot drift from
    // the schema the server expects.
    let store = SqliteStore::open(&target)
        .await
        .map_err(|err| err.to_string())?;

    let owners = owners_by_collection(&source_roles);
    let collections = dedupe_collections(source_collections, &owners);

    for collection in collections.values() {
        store
            .store_collection(collection.clone())
            .await
            .map_err(|err| format!("Failed to write a collection: {err}"))?;
    }
    info!("Wrote {} collections.", collections.len());

    let mut roles_written = 0usize;
    for role in &source_roles {
        let (Some(collection_id), Some(user_id)) =
            (parse_id(&role.collection_id), parse_id(&role.principal_id))
        else {
            warn!(
                "Skipping a role assignment with unparseable keys ({}/{}).",
                role.collection_id, role.principal_id
            );
            continue;
        };

        if !collections.contains_key(&collection_id) {
            // Table Storage had no foreign keys, so a role could outlive the
            // collection it referred to.
            warn!(
                "Skipping a role assignment for collection {} because that collection has no row.",
                role.collection_id
            );
            continue;
        }

        let parsed = Role::from(role.role.as_str());
        if parsed == Role::Invalid {
            warn!(
                "Skipping a role assignment with an unrecognised role {:?}.",
                role.role
            );
            continue;
        }

        store
            .store_role_assignment(RoleAssignment {
                collection_id,
                user_id,
                role: parsed,
            })
            .await
            .map_err(|err| format!("Failed to write a role assignment: {err}"))?;

        roles_written += 1;
    }
    info!("Wrote {roles_written} role assignments.");

    let mut ideas_written = 0usize;
    for idea in &source_ideas {
        let (Some(collection_id), Some(id)) = (parse_id(&idea.collection_id), parse_id(&idea.id))
        else {
            warn!(
                "Skipping an idea with unparseable keys ({}/{}).",
                idea.collection_id, idea.id
            );
            continue;
        };

        if !collections.contains_key(&collection_id) {
            warn!(
                "Skipping idea {} because collection {} has no row.",
                idea.id, idea.collection_id
            );
            continue;
        }

        store
            .store_idea(Idea {
                id,
                collection_id,
                name: idea.name.clone(),
                description: idea.description.clone(),
                tags: split_tags(&idea.tags),
                completed: idea.completed,
            })
            .await
            .map_err(|err| format!("Failed to write an idea: {err}"))?;

        ideas_written += 1;
    }
    info!("Wrote {ideas_written} ideas.");

    let mut users_written = 0usize;
    for user in &source_users {
        let (Some(email_hash), Some(principal_id)) =
            (parse_id(&user.email_hash), parse_id(&user.principal_id))
        else {
            warn!("Skipping a user with unparseable keys.");
            continue;
        };

        store
            .store_user(User {
                principal_id,
                email_hash,
                first_name: user.first_name.clone(),
            })
            .await
            .map_err(|err| format!("Failed to write a user: {err}"))?;

        users_written += 1;
    }
    info!("Wrote {users_written} users.");

    // Read back what we wrote, so a silent write failure cannot pass for a
    // successful import.
    verify(&store, &collections, ideas_written).await?;

    info!("Import complete: {target}");

    Ok(())
}

/// The old layout duplicated a collection's row into every member's partition,
/// so a rename by the owner after sharing left the copies disagreeing.
///
/// The owner's copy wins, and every name that loses is logged so it can be
/// reviewed before go-live.
fn dedupe_collections(
    source: Vec<TableCollection>,
    owners: &HashMap<u128, u128>,
) -> HashMap<u128, Collection> {
    let mut collections: HashMap<u128, Collection> = HashMap::new();

    for entity in source {
        let (Some(collection_id), Some(principal_id)) = (
            parse_id(&entity.collection_id),
            parse_id(&entity.principal_id),
        ) else {
            warn!(
                "Skipping a collection with unparseable keys ({}/{}).",
                entity.principal_id, entity.collection_id
            );
            continue;
        };

        let from_owner = owners.get(&collection_id) == Some(&principal_id);

        match collections.get(&collection_id) {
            Some(existing) if existing.name == entity.name => {}
            Some(existing) if from_owner => {
                warn!(
                    "Collection {} has diverging names; keeping the owner's {:?} over {:?}.",
                    format_id(collection_id),
                    entity.name,
                    existing.name
                );
            }
            Some(existing) => {
                warn!(
                    "Collection {} has diverging names; keeping {:?} over {:?}.",
                    format_id(collection_id),
                    existing.name,
                    entity.name
                );
                continue;
            }
            None => {}
        }

        collections.insert(
            collection_id,
            Collection {
                collection_id,
                user_id: principal_id,
                name: entity.name,
            },
        );
    }

    collections
}

fn owners_by_collection(roles: &[TableRoleAssignment]) -> HashMap<u128, u128> {
    roles
        .iter()
        .filter(|role| Role::from(role.role.as_str()) == Role::Owner)
        .filter_map(|role| {
            Some((
                parse_id(&role.collection_id)?,
                parse_id(&role.principal_id)?,
            ))
        })
        .collect()
}

/// Tags were stored comma-joined, historically with a leading comma.
///
/// Splitting is lossy only for a tag containing a comma, which the interface
/// has never been able to produce.
fn split_tags(raw: &str) -> HashSet<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(str::to_string)
        .collect()
}

async fn verify(
    store: &SqliteStore,
    collections: &HashMap<u128, Collection>,
    expected_ideas: usize,
) -> Result<(), String> {
    let mut counted = 0usize;

    for id in collections.keys() {
        counted += store
            .get_ideas(*id, Default::default())
            .await
            .map_err(|err| format!("Failed to read back collection {}: {err}", format_id(*id)))?
            .len();
    }

    if counted != expected_ideas {
        error!("Wrote {expected_ideas} ideas but read back {counted}.");
        return Err("The import did not round-trip; do not use this database.".into());
    }

    info!(
        "Verified {counted} ideas across {} collections.",
        collections.len()
    );

    Ok(())
}

async fn read_all<T>(tables: &TableServiceClient, table: &str) -> Result<Vec<T>, String>
where
    T: serde::de::DeserializeOwned + Clone + Send + Sync,
{
    let client = tables.table_client(table);
    let mut entities = Vec::new();
    let mut stream = Box::pin(client.query().into_stream::<T>());

    while let Some(page) = stream.next().await {
        let mut page = page.map_err(|err| format!("Failed to read the {table} table: {err}"))?;
        entities.append(&mut page.entities);
    }

    Ok(entities)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_the_historical_leading_comma() {
        assert_eq!(
            split_tags(",one,two"),
            ["one", "two"].iter().map(|t| t.to_string()).collect()
        );
    }

    #[test]
    fn ignores_empty_tag_strings() {
        assert!(split_tags("").is_empty());
        assert!(split_tags(",").is_empty());
    }

    #[test]
    fn keeps_the_owners_name_when_copies_diverge() {
        let owners = HashMap::from([(1u128, 7u128)]);

        let collections = dedupe_collections(
            vec![
                TableCollection {
                    principal_id: format_id(9),
                    collection_id: format_id(1),
                    name: "Stale copy".into(),
                },
                TableCollection {
                    principal_id: format_id(7),
                    collection_id: format_id(1),
                    name: "Owner's name".into(),
                },
            ],
            &owners,
        );

        assert_eq!(collections.len(), 1);
        assert_eq!(collections[&1].name, "Owner's name");
    }

    #[test]
    fn a_members_copy_does_not_overwrite_the_owners() {
        let owners = HashMap::from([(1u128, 7u128)]);

        let collections = dedupe_collections(
            vec![
                TableCollection {
                    principal_id: format_id(7),
                    collection_id: format_id(1),
                    name: "Owner's name".into(),
                },
                TableCollection {
                    principal_id: format_id(9),
                    collection_id: format_id(1),
                    name: "Stale copy".into(),
                },
            ],
            &owners,
        );

        assert_eq!(collections[&1].name, "Owner's name");
    }
}
