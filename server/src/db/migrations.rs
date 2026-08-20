//! Schema migrations, applied in order and tracked in the `migrations` table.
//!
//! Migrations are append-only: never edit one that has shipped, add another.

/// Each entry is one migration, applied inside its own transaction. The index
/// of an entry (plus one) is its version number.
pub const MIGRATIONS: &[&str] = &[
    // 1 — the initial schema.
    r#"
    CREATE TABLE users (
        principal_id TEXT PRIMARY KEY,          -- 32-char zero-padded lowercase hex
        email_hash   TEXT NOT NULL UNIQUE,      -- md5(lowercase(trimmed email)), hex
        first_name   TEXT NOT NULL
    );

    CREATE TABLE collections (
        id   TEXT PRIMARY KEY,
        name TEXT NOT NULL
    );

    CREATE TABLE role_assignments (
        collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
        principal_id  TEXT NOT NULL,
        role          TEXT NOT NULL CHECK (role IN ('Owner','Contributor','Viewer')),
        PRIMARY KEY (collection_id, principal_id)
    );

    CREATE INDEX role_assignments_by_principal ON role_assignments(principal_id);

    CREATE TABLE ideas (
        collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
        id            TEXT NOT NULL,
        name          TEXT NOT NULL,
        description   TEXT NOT NULL,
        completed     INTEGER NOT NULL DEFAULT 0,
        PRIMARY KEY (collection_id, id)
    );

    CREATE TABLE idea_tags (
        collection_id TEXT NOT NULL,
        idea_id       TEXT NOT NULL,
        tag           TEXT NOT NULL,
        PRIMARY KEY (collection_id, idea_id, tag),
        FOREIGN KEY (collection_id, idea_id)
            REFERENCES ideas(collection_id, id) ON DELETE CASCADE
    );

    CREATE INDEX idea_tags_by_tag ON idea_tags(collection_id, tag);
    "#,
];
