//! One contract, two stores.
//!
//! Handler tests run against [`MemoryStore`] because it is fast and needs no
//! setup, which is only safe while the two stores behave identically. Every
//! case below therefore runs twice — once in memory, once against a real
//! SQLite database — so any drift in ordering, not-found codes, upsert
//! semantics, or filtering fails immediately rather than in production.

use std::collections::HashSet;

use super::{MemoryStore, SqliteStore, Store};
use crate::models::*;

/// Generates a test per case per store.
macro_rules! conformance {
    ($($name:ident),* $(,)?) => {
        mod memory {
            $(
                #[actix_rt::test]
                async fn $name() {
                    super::$name(super::MemoryStore::new()).await;
                }
            )*
        }

        mod sqlite {
            $(
                #[actix_rt::test]
                async fn $name() {
                    let store = super::SqliteStore::open_in_memory()
                        .await
                        .expect("an in-memory database should open");

                    super::$name(store).await;
                }
            )*
        }
    };
}

conformance!(
    health_is_ok,
    ideas_round_trip,
    get_idea_reports_a_missing_collection,
    get_idea_reports_a_missing_idea,
    store_idea_requires_its_collection,
    ideas_are_ordered_by_id,
    ideas_filter_by_completion,
    ideas_filter_by_tag_exactly,
    random_idea_respects_filters,
    random_idea_reports_an_empty_collection,
    remove_idea_reports_a_missing_idea,
    collections_round_trip,
    get_collection_requires_a_role,
    get_collections_reports_a_principal_with_none,
    a_shared_collection_is_one_row,
    owner_removing_a_collection_removes_it_for_everyone,
    member_removing_a_collection_only_leaves_it,
    role_assignments_round_trip,
    get_role_assignment_denies_a_stranger,
    role_assignments_require_their_collection,
    remove_role_assignment_reports_a_missing_principal,
    users_round_trip,
    get_user_reports_a_missing_hash,
);

// ── Fixtures ─────────────────────────────────────────────────────────────────

const OWNER: Id = 1;
const MEMBER: Id = 2;
const STRANGER: Id = 3;
const COLLECTION: Id = 10;

async fn seed_collection<S: Store>(store: &S, id: Id, owner: Id, name: &str) {
    store
        .store_collection(Collection {
            collection_id: id,
            user_id: owner,
            name: name.into(),
        })
        .await
        .expect("the collection should store");

    store
        .store_role_assignment(RoleAssignment {
            collection_id: id,
            user_id: owner,
            role: Role::Owner,
        })
        .await
        .expect("the owner's role should store");
}

fn tags(values: &[&str]) -> HashSet<String> {
    values.iter().map(|t| t.to_string()).collect()
}

fn idea(id: Id, collection: Id, name: &str, tags: HashSet<String>) -> Idea {
    Idea {
        id,
        collection_id: collection,
        name: name.into(),
        description: format!("The description of {name}."),
        tags,
        completed: false,
    }
}

// ── Cases ────────────────────────────────────────────────────────────────────

async fn health_is_ok<S: Store>(store: S) {
    let health = store.health().await.expect("health should be readable");
    assert!(health.ok);
}

async fn ideas_round_trip<S: Store>(store: S) {
    seed_collection(&store, COLLECTION, OWNER, "Ideas").await;

    let stored = store
        .store_idea(idea(1, COLLECTION, "Learn Rust", tags(&["learning"])))
        .await
        .expect("the idea should store");

    assert_eq!(stored.name, "Learn Rust");

    let read = store
        .get_idea(COLLECTION, 1)
        .await
        .expect("the idea should be readable");

    assert_eq!(read, stored);

    // Storing the same id again is an update, not a duplicate.
    store
        .store_idea(Idea {
            completed: true,
            tags: tags(&["learning", "done"]),
            ..stored
        })
        .await
        .expect("the idea should update");

    let read = store.get_idea(COLLECTION, 1).await.expect("still readable");
    assert!(read.completed);
    assert_eq!(read.tags, tags(&["learning", "done"]));

    assert_eq!(
        store
            .get_ideas(COLLECTION, IdeaFilter::default())
            .await
            .expect("the collection should list")
            .len(),
        1
    );

    store
        .remove_idea(COLLECTION, 1)
        .await
        .expect("the idea should remove");

    assert_eq!(
        store.get_idea(COLLECTION, 1).await.unwrap_err().code,
        404,
        "a removed idea should be gone"
    );
}

async fn get_idea_reports_a_missing_collection<S: Store>(store: S) {
    assert_eq!(store.get_idea(COLLECTION, 1).await.unwrap_err().code, 404);
    assert_eq!(
        store
            .get_ideas(COLLECTION, IdeaFilter::default())
            .await
            .unwrap_err()
            .code,
        404
    );
}

async fn get_idea_reports_a_missing_idea<S: Store>(store: S) {
    seed_collection(&store, COLLECTION, OWNER, "Ideas").await;

    let err = store.get_idea(COLLECTION, 404).await.unwrap_err();
    assert_eq!(err.code, 404);
}

async fn store_idea_requires_its_collection<S: Store>(store: S) {
    let err = store
        .store_idea(idea(1, COLLECTION, "Orphan", HashSet::new()))
        .await
        .unwrap_err();

    assert_eq!(
        err.code, 404,
        "an idea cannot belong to a collection that does not exist"
    );
}

async fn ideas_are_ordered_by_id<S: Store>(store: S) {
    seed_collection(&store, COLLECTION, OWNER, "Ideas").await;

    for id in [3, 1, 2] {
        store
            .store_idea(idea(id, COLLECTION, &format!("Idea {id}"), HashSet::new()))
            .await
            .expect("the idea should store");
    }

    let ideas = store
        .get_ideas(COLLECTION, IdeaFilter::default())
        .await
        .expect("the collection should list");

    assert_eq!(
        ideas.iter().map(|i| i.id).collect::<Vec<_>>(),
        vec![1, 2, 3],
        "both stores must agree on ordering"
    );
}

async fn ideas_filter_by_completion<S: Store>(store: S) {
    seed_collection(&store, COLLECTION, OWNER, "Ideas").await;

    store
        .store_idea(idea(1, COLLECTION, "Open", HashSet::new()))
        .await
        .unwrap();
    store
        .store_idea(Idea {
            completed: true,
            ..idea(2, COLLECTION, "Done", HashSet::new())
        })
        .await
        .unwrap();

    let open = store
        .get_ideas(COLLECTION, IdeaFilter::new(None, Some(false)))
        .await
        .unwrap();
    assert_eq!(open.iter().map(|i| i.id).collect::<Vec<_>>(), vec![1]);

    let done = store
        .get_ideas(COLLECTION, IdeaFilter::new(None, Some(true)))
        .await
        .unwrap();
    assert_eq!(done.iter().map(|i| i.id).collect::<Vec<_>>(), vec![2]);
}

/// The regression test for the old comma-joined tag column, where filtering for
/// `rust` also matched an idea tagged `rustacean`.
async fn ideas_filter_by_tag_exactly<S: Store>(store: S) {
    seed_collection(&store, COLLECTION, OWNER, "Ideas").await;

    store
        .store_idea(idea(1, COLLECTION, "Tagged", tags(&["rust"])))
        .await
        .unwrap();
    store
        .store_idea(idea(2, COLLECTION, "Similar", tags(&["rustacean"])))
        .await
        .unwrap();

    let matched = store
        .get_ideas(COLLECTION, IdeaFilter::new(Some("rust".into()), None))
        .await
        .unwrap();

    assert_eq!(
        matched.iter().map(|i| i.id).collect::<Vec<_>>(),
        vec![1],
        "a tag filter must not match a longer tag that starts with it"
    );
}

async fn random_idea_respects_filters<S: Store>(store: S) {
    seed_collection(&store, COLLECTION, OWNER, "Ideas").await;

    store
        .store_idea(idea(1, COLLECTION, "Wanted", tags(&["wanted"])))
        .await
        .unwrap();
    store
        .store_idea(idea(2, COLLECTION, "Unwanted", tags(&["unwanted"])))
        .await
        .unwrap();

    // Random or not, the filter has to hold every time.
    for _ in 0..20 {
        let chosen = store
            .get_random_idea(COLLECTION, IdeaFilter::new(Some("wanted".into()), None))
            .await
            .expect("a matching idea should be chosen");

        assert_eq!(chosen.id, 1);
    }
}

async fn random_idea_reports_an_empty_collection<S: Store>(store: S) {
    seed_collection(&store, COLLECTION, OWNER, "Ideas").await;

    let err = store
        .get_random_idea(COLLECTION, IdeaFilter::default())
        .await
        .unwrap_err();

    assert_eq!(err.code, 404);
}

async fn remove_idea_reports_a_missing_idea<S: Store>(store: S) {
    seed_collection(&store, COLLECTION, OWNER, "Ideas").await;

    assert_eq!(
        store.remove_idea(COLLECTION, 404).await.unwrap_err().code,
        404
    );
}

async fn collections_round_trip<S: Store>(store: S) {
    seed_collection(&store, COLLECTION, OWNER, "Ideas").await;

    let read = store
        .get_collection(COLLECTION, OWNER)
        .await
        .expect("the owner should see their collection");

    assert_eq!(read.name, "Ideas");
    assert_eq!(
        read.user_id, OWNER,
        "the collection is reported as belonging to whoever asked for it"
    );

    store
        .store_collection(Collection {
            collection_id: COLLECTION,
            user_id: OWNER,
            name: "Renamed".into(),
        })
        .await
        .expect("the collection should rename");

    assert_eq!(
        store.get_collection(COLLECTION, OWNER).await.unwrap().name,
        "Renamed"
    );

    let all = store
        .get_collections(OWNER)
        .await
        .expect("the owner should have collections");
    assert_eq!(all.len(), 1);
}

async fn get_collection_requires_a_role<S: Store>(store: S) {
    seed_collection(&store, COLLECTION, OWNER, "Ideas").await;

    let err = store
        .get_collection(COLLECTION, STRANGER)
        .await
        .unwrap_err();
    assert_eq!(
        err.code, 404,
        "a collection you hold no role on should look like one that does not exist"
    );
}

async fn get_collections_reports_a_principal_with_none<S: Store>(store: S) {
    let err = store.get_collections(STRANGER).await.unwrap_err();
    assert_eq!(err.code, 404);
}

/// Sharing is a role assignment and nothing else, so a later rename by the
/// owner reaches every member — the bug the old per-member copies caused.
async fn a_shared_collection_is_one_row<S: Store>(store: S) {
    seed_collection(&store, COLLECTION, OWNER, "Ideas").await;

    store
        .store_role_assignment(RoleAssignment {
            collection_id: COLLECTION,
            user_id: MEMBER,
            role: Role::Viewer,
        })
        .await
        .expect("the member's role should store");

    assert_eq!(
        store.get_collection(COLLECTION, MEMBER).await.unwrap().name,
        "Ideas"
    );

    store
        .store_collection(Collection {
            collection_id: COLLECTION,
            user_id: OWNER,
            name: "Renamed".into(),
        })
        .await
        .unwrap();

    assert_eq!(
        store.get_collection(COLLECTION, MEMBER).await.unwrap().name,
        "Renamed",
        "a rename by the owner should be visible to every member"
    );
}

async fn owner_removing_a_collection_removes_it_for_everyone<S: Store>(store: S) {
    seed_collection(&store, COLLECTION, OWNER, "Ideas").await;

    store
        .store_role_assignment(RoleAssignment {
            collection_id: COLLECTION,
            user_id: MEMBER,
            role: Role::Viewer,
        })
        .await
        .unwrap();

    store
        .store_idea(idea(1, COLLECTION, "Doomed", HashSet::new()))
        .await
        .unwrap();

    store
        .remove_collection(COLLECTION, OWNER)
        .await
        .expect("the owner should be able to remove it");

    assert_eq!(
        store
            .get_collection(COLLECTION, MEMBER)
            .await
            .unwrap_err()
            .code,
        404
    );
    assert_eq!(store.get_idea(COLLECTION, 1).await.unwrap_err().code, 404);
    assert_eq!(
        store
            .get_role_assignment(COLLECTION, MEMBER)
            .await
            .unwrap_err()
            .code,
        403
    );
}

async fn member_removing_a_collection_only_leaves_it<S: Store>(store: S) {
    seed_collection(&store, COLLECTION, OWNER, "Ideas").await;

    store
        .store_role_assignment(RoleAssignment {
            collection_id: COLLECTION,
            user_id: MEMBER,
            role: Role::Viewer,
        })
        .await
        .unwrap();

    store
        .remove_collection(COLLECTION, MEMBER)
        .await
        .expect("a member should be able to leave");

    assert_eq!(
        store
            .get_collection(COLLECTION, MEMBER)
            .await
            .unwrap_err()
            .code,
        404
    );
    assert_eq!(
        store.get_collection(COLLECTION, OWNER).await.unwrap().name,
        "Ideas",
        "one member leaving must not remove the collection"
    );
}

async fn role_assignments_round_trip<S: Store>(store: S) {
    seed_collection(&store, COLLECTION, OWNER, "Ideas").await;

    store
        .store_role_assignment(RoleAssignment {
            collection_id: COLLECTION,
            user_id: MEMBER,
            role: Role::Viewer,
        })
        .await
        .unwrap();

    // Re-assigning is an update.
    store
        .store_role_assignment(RoleAssignment {
            collection_id: COLLECTION,
            user_id: MEMBER,
            role: Role::Contributor,
        })
        .await
        .unwrap();

    assert_eq!(
        store
            .get_role_assignment(COLLECTION, MEMBER)
            .await
            .unwrap()
            .role,
        Role::Contributor
    );

    let all = store.get_role_assignments(COLLECTION).await.unwrap();
    assert_eq!(
        all.iter().map(|r| r.user_id).collect::<Vec<_>>(),
        vec![OWNER, MEMBER],
        "both stores must agree on ordering"
    );

    store
        .remove_role_assignment(COLLECTION, MEMBER)
        .await
        .unwrap();

    assert_eq!(
        store
            .get_role_assignment(COLLECTION, MEMBER)
            .await
            .unwrap_err()
            .code,
        403
    );
}

async fn get_role_assignment_denies_a_stranger<S: Store>(store: S) {
    seed_collection(&store, COLLECTION, OWNER, "Ideas").await;

    let err = store
        .get_role_assignment(COLLECTION, STRANGER)
        .await
        .unwrap_err();

    assert_eq!(
        err.code, 403,
        "the absence of a role is a refusal, not a missing resource"
    );
}

async fn role_assignments_require_their_collection<S: Store>(store: S) {
    assert_eq!(
        store
            .store_role_assignment(RoleAssignment {
                collection_id: COLLECTION,
                user_id: MEMBER,
                role: Role::Viewer,
            })
            .await
            .unwrap_err()
            .code,
        404
    );

    assert_eq!(
        store
            .get_role_assignments(COLLECTION)
            .await
            .unwrap_err()
            .code,
        404
    );

    assert_eq!(
        store
            .remove_role_assignment(COLLECTION, MEMBER)
            .await
            .unwrap_err()
            .code,
        404
    );
}

async fn remove_role_assignment_reports_a_missing_principal<S: Store>(store: S) {
    seed_collection(&store, COLLECTION, OWNER, "Ideas").await;

    assert_eq!(
        store
            .remove_role_assignment(COLLECTION, STRANGER)
            .await
            .unwrap_err()
            .code,
        404
    );
}

async fn users_round_trip<S: Store>(store: S) {
    let user = User {
        principal_id: OWNER,
        email_hash: email_hash("owner@example.com"),
        first_name: "Owner".into(),
    };

    store.store_user(user.clone()).await.unwrap();

    assert_eq!(
        store
            .get_user(email_hash("owner@example.com"))
            .await
            .unwrap(),
        user
    );

    // Signing in again updates rather than duplicating.
    let renamed = User {
        first_name: "Renamed".into(),
        ..user
    };
    store.store_user(renamed.clone()).await.unwrap();

    assert_eq!(
        store
            .get_user(email_hash("owner@example.com"))
            .await
            .unwrap()
            .first_name,
        "Renamed"
    );
}

async fn get_user_reports_a_missing_hash<S: Store>(store: S) {
    assert_eq!(
        store
            .get_user(email_hash("nobody@example.com"))
            .await
            .unwrap_err()
            .code,
        404
    );
}
