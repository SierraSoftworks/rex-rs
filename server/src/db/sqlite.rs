use std::{collections::HashSet, path::Path, sync::Arc};

use rex_api::ApiError;
use rusqlite::{OptionalExtension, params};
use tracing::{error, instrument};

use super::{
    MIGRATIONS, Store, collection_not_found, idea_not_found, no_access, no_random_idea,
    principal_not_found, user_not_found,
};
use crate::models::*;

/// The production [`Store`]: one shared SQLite connection on its own thread.
///
/// SQLite has a single writer, which is why the deployment runs one replica
/// with `strategy: Recreate`. At Rex's scale WAL plus a busy timeout is more
/// than enough, and the payoff is a binary with no external dependencies.
#[derive(Clone)]
pub struct SqliteStore {
    conn: Arc<tokio_rusqlite::Connection>,
    started_at: chrono::DateTime<chrono::Utc>,
}

impl SqliteStore {
    /// Opens (creating if necessary) the database at `path` and brings it up to
    /// the latest schema version.
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self, ApiError> {
        let conn = tokio_rusqlite::Connection::open(path.as_ref())
            .await
            .map_err(|err| startup_error("open the database", err))?;

        Self::prepare(conn, false, MIGRATIONS.len()).await
    }

    /// A throwaway database, used by the conformance and migration suites.
    pub async fn open_in_memory() -> Result<Self, ApiError> {
        Self::open_in_memory_at_migration(MIGRATIONS.len()).await
    }

    /// A throwaway database stopped part-way through the migration list, so a
    /// test can seed data at an old schema version and then migrate the rest of
    /// the way.
    pub async fn open_in_memory_at_migration(version: usize) -> Result<Self, ApiError> {
        let conn = tokio_rusqlite::Connection::open_in_memory()
            .await
            .map_err(|err| startup_error("open an in-memory database", err))?;

        Self::prepare(conn, true, version).await
    }

    async fn prepare(
        conn: tokio_rusqlite::Connection,
        in_memory: bool,
        up_to: usize,
    ) -> Result<Self, ApiError> {
        conn.call(move |conn| {
            conn.busy_timeout(std::time::Duration::from_secs(5))?;
            conn.pragma_update(None, "foreign_keys", "ON")?;

            if !in_memory {
                // WAL is a property of the file, so it only needs setting once,
                // but asking for it is cheap and idempotent. `synchronous =
                // NORMAL` is only safe once WAL actually took.
                let mode: String =
                    conn.pragma_update_and_check(None, "journal_mode", "WAL", |row| row.get(0))?;

                if mode.eq_ignore_ascii_case("wal") {
                    conn.pragma_update(None, "synchronous", "NORMAL")?;
                }
            }

            migrate(conn, up_to)
        })
        .await
        .map_err(|err| startup_error("prepare the database", err))?;

        Ok(Self {
            conn: Arc::new(conn),
            started_at: chrono::Utc::now(),
        })
    }

    /// Runs `f` on the connection thread, translating transport failures into a
    /// 500 while letting the closure's own [`ApiError`]s through untouched.
    async fn call<T, F>(&self, what: &'static str, f: F) -> Result<T, ApiError>
    where
        F: FnOnce(&mut rusqlite::Connection) -> Result<T, ApiError> + Send + 'static,
        T: Send + 'static,
    {
        match self.conn.call(f).await {
            Ok(value) => Ok(value),
            Err(tokio_rusqlite::Error::Error(err)) => Err(err),
            Err(err) => {
                error!({ exception.message = %err }, "Unable to {} because the database connection failed.", what);
                Err(ApiError::internal_server_error())
            }
        }
    }
}

fn startup_error(what: &str, err: impl std::fmt::Display) -> ApiError {
    error!({ exception.message = %err }, "Unable to {}.", what);
    ApiError::internal_server_error()
}

/// Turns a rusqlite failure into the generic 500, logging the detail rather
/// than leaking it to the caller.
fn db_error(err: rusqlite::Error) -> ApiError {
    error!({ exception.message = %err }, "We were unable to query the database.");
    ApiError::internal_server_error()
}

fn migrate(conn: &mut rusqlite::Connection, up_to: usize) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS migrations (
            version    INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );",
    )?;

    let current: usize = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM migrations",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    for (index, migration) in MIGRATIONS.iter().enumerate().take(up_to) {
        let version = index + 1;
        if version <= current {
            continue;
        }

        let tx = conn.transaction()?;
        tx.execute_batch(migration)?;
        tx.execute(
            "INSERT INTO migrations (version, applied_at) VALUES (?1, ?2)",
            params![version, chrono::Utc::now().to_rfc3339()],
        )?;
        tx.commit()?;
    }

    Ok(())
}

// ── Row helpers ──────────────────────────────────────────────────────────────

fn collection_exists(conn: &rusqlite::Connection, id: &str) -> Result<bool, ApiError> {
    conn.query_row(
        "SELECT 1 FROM collections WHERE id = ?1",
        params![id],
        |_| Ok(()),
    )
    .optional()
    .map(|found| found.is_some())
    .map_err(db_error)
}

fn require_collection(conn: &rusqlite::Connection, id: &str) -> Result<(), ApiError> {
    if collection_exists(conn, id)? {
        Ok(())
    } else {
        Err(collection_not_found())
    }
}

fn load_tags(
    conn: &rusqlite::Connection,
    collection: &str,
    idea: &str,
) -> Result<HashSet<String>, ApiError> {
    let mut stmt = conn
        .prepare_cached(
            "SELECT tag FROM idea_tags WHERE collection_id = ?1 AND idea_id = ?2 ORDER BY tag",
        )
        .map_err(db_error)?;

    let tags = stmt
        .query_map(params![collection, idea], |row| row.get::<_, String>(0))
        .map_err(db_error)?
        .collect::<Result<HashSet<String>, _>>()
        .map_err(db_error)?;

    Ok(tags)
}

fn write_tags(
    conn: &rusqlite::Connection,
    collection: &str,
    idea: &str,
    tags: &HashSet<String>,
) -> Result<(), ApiError> {
    conn.execute(
        "DELETE FROM idea_tags WHERE collection_id = ?1 AND idea_id = ?2",
        params![collection, idea],
    )
    .map_err(db_error)?;

    for tag in tags {
        conn.execute(
            "INSERT INTO idea_tags (collection_id, idea_id, tag) VALUES (?1, ?2, ?3)",
            params![collection, idea, tag],
        )
        .map_err(db_error)?;
    }

    Ok(())
}

/// The `WHERE` fragment shared by the filtered idea queries.
///
/// Both parameters are nullable, and a null one disables its clause, which
/// keeps a single prepared statement covering every combination of filters.
const IDEA_FILTER: &str = "
    AND (?2 IS NULL OR i.completed = ?2)
    AND (?3 IS NULL OR EXISTS (
        SELECT 1 FROM idea_tags t
        WHERE t.collection_id = i.collection_id AND t.idea_id = i.id AND t.tag = ?3
    ))";

fn read_idea_row(row: &rusqlite::Row<'_>, collection: Id) -> rusqlite::Result<Idea> {
    Ok(Idea {
        id: parse_id(&row.get::<_, String>(0)?).unwrap_or_default(),
        collection_id: collection,
        name: row.get(1)?,
        description: row.get(2)?,
        tags: HashSet::new(),
        completed: row.get::<_, i64>(3)? != 0,
    })
}

impl Store for SqliteStore {
    #[instrument(name = "store.get_idea", skip(self), err, fields(db.system = "sqlite", db.operation = "get_idea"))]
    async fn get_idea(&self, collection: Id, id: Id) -> Result<Idea, ApiError> {
        let (cid, iid) = (format_id(collection), format_id(id));

        self.call("read an idea", move |conn| {
            require_collection(conn, &cid)?;

            let mut idea = conn
                .query_row(
                    "SELECT id, name, description, completed FROM ideas
                     WHERE collection_id = ?1 AND id = ?2",
                    params![cid, iid],
                    |row| read_idea_row(row, collection),
                )
                .optional()
                .map_err(db_error)?
                .ok_or_else(idea_not_found)?;

            idea.tags = load_tags(conn, &cid, &iid)?;
            Ok(idea)
        })
        .await
    }

    #[instrument(name = "store.get_ideas", skip(self), err, fields(db.system = "sqlite", db.operation = "get_ideas"))]
    async fn get_ideas(&self, collection: Id, filter: IdeaFilter) -> Result<Vec<Idea>, ApiError> {
        let cid = format_id(collection);

        self.call("list ideas", move |conn| {
            require_collection(conn, &cid)?;

            let sql = format!(
                "SELECT i.id, i.name, i.description, i.completed FROM ideas i
                 WHERE i.collection_id = ?1{IDEA_FILTER}
                 ORDER BY i.id"
            );

            let mut stmt = conn.prepare_cached(&sql).map_err(db_error)?;
            let mut ideas = stmt
                .query_map(
                    params![cid, filter.is_completed.map(i64::from), filter.tag],
                    |row| read_idea_row(row, collection),
                )
                .map_err(db_error)?
                .collect::<Result<Vec<Idea>, _>>()
                .map_err(db_error)?;

            drop(stmt);

            for idea in ideas.iter_mut() {
                idea.tags = load_tags(conn, &cid, &format_id(idea.id))?;
            }

            Ok(ideas)
        })
        .await
    }

    #[instrument(name = "store.get_random_idea", skip(self), err, fields(db.system = "sqlite", db.operation = "get_random_idea"))]
    async fn get_random_idea(&self, collection: Id, filter: IdeaFilter) -> Result<Idea, ApiError> {
        let cid = format_id(collection);

        self.call("read a random idea", move |conn| {
            require_collection(conn, &cid)?;

            // Letting SQLite pick means we never load the whole collection just
            // to throw all but one row away.
            let sql = format!(
                "SELECT i.id, i.name, i.description, i.completed FROM ideas i
                 WHERE i.collection_id = ?1{IDEA_FILTER}
                 ORDER BY RANDOM() LIMIT 1"
            );

            let mut idea = conn
                .query_row(
                    &sql,
                    params![cid, filter.is_completed.map(i64::from), filter.tag],
                    |row| read_idea_row(row, collection),
                )
                .optional()
                .map_err(db_error)?
                .ok_or_else(no_random_idea)?;

            idea.tags = load_tags(conn, &cid, &format_id(idea.id))?;
            Ok(idea)
        })
        .await
    }

    #[instrument(name = "store.store_idea", skip(self), err, fields(db.system = "sqlite", db.operation = "store_idea"))]
    async fn store_idea(&self, idea: Idea) -> Result<Idea, ApiError> {
        let (cid, iid) = (format_id(idea.collection_id), format_id(idea.id));

        self.call("store an idea", move |conn| {
            require_collection(conn, &cid)?;

            let tx = conn.transaction().map_err(db_error)?;
            tx.execute(
                "INSERT INTO ideas (collection_id, id, name, description, completed)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT (collection_id, id) DO UPDATE SET
                     name = excluded.name,
                     description = excluded.description,
                     completed = excluded.completed",
                params![cid, iid, idea.name, idea.description, idea.completed as i64],
            )
            .map_err(db_error)?;

            write_tags(&tx, &cid, &iid, &idea.tags)?;
            tx.commit().map_err(db_error)?;

            Ok(idea)
        })
        .await
    }

    #[instrument(name = "store.remove_idea", skip(self), err, fields(db.system = "sqlite", db.operation = "remove_idea"))]
    async fn remove_idea(&self, collection: Id, id: Id) -> Result<(), ApiError> {
        let (cid, iid) = (format_id(collection), format_id(id));

        self.call("remove an idea", move |conn| {
            require_collection(conn, &cid)?;

            let removed = conn
                .execute(
                    "DELETE FROM ideas WHERE collection_id = ?1 AND id = ?2",
                    params![cid, iid],
                )
                .map_err(db_error)?;

            if removed == 0 {
                Err(idea_not_found())
            } else {
                Ok(())
            }
        })
        .await
    }

    #[instrument(name = "store.get_collection", skip(self), err, fields(db.system = "sqlite", db.operation = "get_collection"))]
    async fn get_collection(&self, id: Id, principal: Id) -> Result<Collection, ApiError> {
        let (cid, pid) = (format_id(id), format_id(principal));

        self.call("read a collection", move |conn| {
            conn.query_row(
                "SELECT c.name FROM collections c
                 JOIN role_assignments r ON r.collection_id = c.id
                 WHERE c.id = ?1 AND r.principal_id = ?2",
                params![cid, pid],
                |row| {
                    Ok(Collection {
                        collection_id: id,
                        user_id: principal,
                        name: row.get(0)?,
                    })
                },
            )
            .optional()
            .map_err(db_error)?
            .ok_or_else(collection_not_found)
        })
        .await
    }

    #[instrument(name = "store.get_collections", skip(self), err, fields(db.system = "sqlite", db.operation = "get_collections"))]
    async fn get_collections(&self, principal: Id) -> Result<Vec<Collection>, ApiError> {
        let pid = format_id(principal);

        self.call("list collections", move |conn| {
            let mut stmt = conn
                .prepare_cached(
                    "SELECT c.id, c.name FROM collections c
                     JOIN role_assignments r ON r.collection_id = c.id
                     WHERE r.principal_id = ?1
                     ORDER BY c.id",
                )
                .map_err(db_error)?;

            let collections = stmt
                .query_map(params![pid], |row| {
                    Ok(Collection {
                        collection_id: parse_id(&row.get::<_, String>(0)?).unwrap_or_default(),
                        user_id: principal,
                        name: row.get(1)?,
                    })
                })
                .map_err(db_error)?
                .collect::<Result<Vec<Collection>, _>>()
                .map_err(db_error)?;

            if collections.is_empty() {
                return Err(principal_not_found());
            }

            Ok(collections)
        })
        .await
    }

    #[instrument(name = "store.store_collection", skip(self), err, fields(db.system = "sqlite", db.operation = "store_collection"))]
    async fn store_collection(&self, collection: Collection) -> Result<Collection, ApiError> {
        let cid = format_id(collection.collection_id);

        self.call("store a collection", move |conn| {
            conn.execute(
                "INSERT INTO collections (id, name) VALUES (?1, ?2)
                 ON CONFLICT (id) DO UPDATE SET name = excluded.name",
                params![cid, collection.name],
            )
            .map_err(db_error)?;

            Ok(collection)
        })
        .await
    }

    #[instrument(name = "store.remove_collection", skip(self), err, fields(db.system = "sqlite", db.operation = "remove_collection"))]
    async fn remove_collection(&self, id: Id, principal: Id) -> Result<(), ApiError> {
        let (cid, pid) = (format_id(id), format_id(principal));

        self.call("remove a collection", move |conn| {
            let role: Role = conn
                .query_row(
                    "SELECT role FROM role_assignments WHERE collection_id = ?1 AND principal_id = ?2",
                    params![cid, pid],
                    |row| row.get::<_, String>(0),
                )
                .optional()
                .map_err(db_error)?
                .ok_or_else(collection_not_found)
                .map(|role| Role::from(role.as_str()))?;

            if role == Role::Owner {
                // Ideas and the other members' role assignments go with it,
                // courtesy of ON DELETE CASCADE.
                conn.execute("DELETE FROM collections WHERE id = ?1", params![cid])
                    .map_err(db_error)?;
            } else {
                conn.execute(
                    "DELETE FROM role_assignments WHERE collection_id = ?1 AND principal_id = ?2",
                    params![cid, pid],
                )
                .map_err(db_error)?;
            }

            Ok(())
        })
        .await
    }

    #[instrument(name = "store.get_role_assignment", skip(self), err, fields(db.system = "sqlite", db.operation = "get_role_assignment"))]
    async fn get_role_assignment(
        &self,
        collection: Id,
        principal: Id,
    ) -> Result<RoleAssignment, ApiError> {
        let (cid, pid) = (format_id(collection), format_id(principal));

        self.call("read a role assignment", move |conn| {
            conn.query_row(
                "SELECT role FROM role_assignments WHERE collection_id = ?1 AND principal_id = ?2",
                params![cid, pid],
                |row| {
                    Ok(RoleAssignment {
                        collection_id: collection,
                        user_id: principal,
                        role: Role::from(row.get::<_, String>(0)?.as_str()),
                    })
                },
            )
            .optional()
            .map_err(db_error)?
            .ok_or_else(no_access)
        })
        .await
    }

    #[instrument(name = "store.get_role_assignments", skip(self), err, fields(db.system = "sqlite", db.operation = "get_role_assignments"))]
    async fn get_role_assignments(&self, collection: Id) -> Result<Vec<RoleAssignment>, ApiError> {
        let cid = format_id(collection);

        self.call("list role assignments", move |conn| {
            require_collection(conn, &cid)?;

            let mut stmt = conn
                .prepare_cached(
                    "SELECT principal_id, role FROM role_assignments
                     WHERE collection_id = ?1
                     ORDER BY principal_id",
                )
                .map_err(db_error)?;

            stmt.query_map(params![cid], |row| {
                Ok(RoleAssignment {
                    collection_id: collection,
                    user_id: parse_id(&row.get::<_, String>(0)?).unwrap_or_default(),
                    role: Role::from(row.get::<_, String>(1)?.as_str()),
                })
            })
            .map_err(db_error)?
            .collect::<Result<Vec<RoleAssignment>, _>>()
            .map_err(db_error)
        })
        .await
    }

    #[instrument(name = "store.store_role_assignment", skip(self), err, fields(db.system = "sqlite", db.operation = "store_role_assignment"))]
    async fn store_role_assignment(
        &self,
        assignment: RoleAssignment,
    ) -> Result<RoleAssignment, ApiError> {
        let (cid, pid) = (
            format_id(assignment.collection_id),
            format_id(assignment.user_id),
        );

        if assignment.role == Role::Invalid {
            return Err(ApiError::bad_request(
                "The role you provided is not one of Owner, Contributor, or Viewer. Please check it and try again.",
            ));
        }

        self.call("store a role assignment", move |conn| {
            require_collection(conn, &cid)?;

            conn.execute(
                "INSERT INTO role_assignments (collection_id, principal_id, role)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT (collection_id, principal_id) DO UPDATE SET role = excluded.role",
                params![cid, pid, String::from(assignment.role)],
            )
            .map_err(db_error)?;

            Ok(assignment)
        })
        .await
    }

    #[instrument(name = "store.remove_role_assignment", skip(self), err, fields(db.system = "sqlite", db.operation = "remove_role_assignment"))]
    async fn remove_role_assignment(&self, collection: Id, principal: Id) -> Result<(), ApiError> {
        let (cid, pid) = (format_id(collection), format_id(principal));

        self.call("remove a role assignment", move |conn| {
            require_collection(conn, &cid)?;

            let removed = conn
                .execute(
                    "DELETE FROM role_assignments WHERE collection_id = ?1 AND principal_id = ?2",
                    params![cid, pid],
                )
                .map_err(db_error)?;

            if removed == 0 {
                Err(principal_not_found())
            } else {
                Ok(())
            }
        })
        .await
    }

    #[instrument(name = "store.get_user", skip(self), err, fields(db.system = "sqlite", db.operation = "get_user"))]
    async fn get_user(&self, email_hash: Id) -> Result<User, ApiError> {
        let hash = format_id(email_hash);

        self.call("read a user", move |conn| {
            conn.query_row(
                "SELECT principal_id, first_name FROM users WHERE email_hash = ?1",
                params![hash],
                |row| {
                    Ok(User {
                        principal_id: parse_id(&row.get::<_, String>(0)?).unwrap_or_default(),
                        email_hash,
                        first_name: row.get(1)?,
                    })
                },
            )
            .optional()
            .map_err(db_error)?
            .ok_or_else(user_not_found)
        })
        .await
    }

    #[instrument(name = "store.store_user", skip(self), err, fields(db.system = "sqlite", db.operation = "store_user"))]
    async fn store_user(&self, user: User) -> Result<User, ApiError> {
        let (pid, hash) = (format_id(user.principal_id), format_id(user.email_hash));

        self.call("store a user", move |conn| {
            // A user is keyed by principal, but looked up by email hash, and a
            // person can change the address they sign in with — so both keys
            // need to converge on the same row.
            conn.execute(
                "INSERT INTO users (principal_id, email_hash, first_name)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT (principal_id) DO UPDATE SET
                     email_hash = excluded.email_hash,
                     first_name = excluded.first_name
                 ON CONFLICT (email_hash) DO UPDATE SET
                     principal_id = excluded.principal_id,
                     first_name = excluded.first_name",
                params![pid, hash, user.first_name],
            )
            .map_err(db_error)?;

            Ok(user)
        })
        .await
    }

    #[instrument(name = "store.health", skip(self), err, fields(db.system = "sqlite", db.operation = "health"))]
    async fn health(&self) -> Result<Health, ApiError> {
        let started_at = self.started_at;

        self.call("check the database", move |conn| {
            // A real query, so that a wedged database fails the readiness probe
            // instead of reporting `ok: true` forever.
            conn.query_row("SELECT 1", [], |row| row.get::<_, i64>(0))
                .map_err(db_error)?;

            Ok(Health {
                ok: true,
                started_at,
            })
        })
        .await
    }
}
