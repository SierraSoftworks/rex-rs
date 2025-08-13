use super::{ensure_user_collection, APIError, AuthToken};
use super::{CollectionIdFilter, IdFilter};
use crate::{models::*, telemetry::TraceMessageExt};
use actix_web::{put, web};
use tracing::instrument;

#[instrument(err, skip(state, token), fields(otel.kind = "internal"))]
#[put("/api/v1/idea/{id}")]
async fn store_idea_v1(
    (info, new_idea, state, token): (
        web::Path<IdFilter>,
        web::Json<IdeaV1>,
        web::Data<GlobalState>,
        AuthToken,
    ),
) -> Result<IdeaV1, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Write");

    let idea: Idea = new_idea.into_inner().into();
    let id = parse_uuid!(info.id, "idea ID");
    let uid = parse_uuid!(token.oid(), "auth token oid");

    ensure_user_collection(&state, &token).await?;

    state
        .store
        .send(
            StoreIdea {
                id,
                collection: uid,
                name: idea.name,
                description: idea.description,
                tags: idea.tags,
                completed: idea.completed,
            }
            .trace(),
        )
        .await?
        .map(|idea| idea.into())
}

#[instrument(err, skip(state, token), fields(otel.kind = "internal"))]
#[put("/api/v2/idea/{id}")]
async fn store_idea_v2(
    (info, new_idea, state, token): (
        web::Path<IdFilter>,
        web::Json<IdeaV2>,
        web::Data<GlobalState>,
        AuthToken,
    ),
) -> Result<IdeaV2, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Write");

    let idea: Idea = new_idea.into_inner().into();
    let id = parse_uuid!(info.id, "idea ID");
    let uid = parse_uuid!(token.oid(), "auth token oid");

    ensure_user_collection(&state, &token).await?;

    state
        .store
        .send(
            StoreIdea {
                id,
                collection: uid,
                name: idea.name,
                description: idea.description,
                tags: idea.tags,
                completed: idea.completed,
            }
            .trace(),
        )
        .await?
        .map(|idea| idea.into())
}

#[instrument(err, skip(state, token), fields(otel.kind = "internal"))]
#[put("/api/v3/idea/{id}")]
async fn store_idea_v3(
    (info, new_idea, state, token): (
        web::Path<IdFilter>,
        web::Json<IdeaV3>,
        web::Data<GlobalState>,
        AuthToken,
    ),
) -> Result<IdeaV3, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Write");

    let idea: Idea = new_idea.into_inner().into();
    let id = parse_uuid!(info.id, "idea ID");
    let uid = parse_uuid!(token.oid(), "auth token oid");

    ensure_user_collection(&state, &token).await?;

    state
        .store
        .send(
            StoreIdea {
                id,
                collection: uid,
                name: idea.name,
                description: idea.description,
                tags: idea.tags,
                completed: idea.completed,
            }
            .trace(),
        )
        .await?
        .map(|idea| idea.into())
}

#[instrument(err, skip(state, token), fields(otel.kind = "internal"))]
#[put("/api/v3/collection/{collection}/idea/{id}")]
async fn store_collection_idea_v3(
    (info, new_idea, state, token): (
        web::Path<CollectionIdFilter>,
        web::Json<IdeaV3>,
        web::Data<GlobalState>,
        AuthToken,
    ),
) -> Result<IdeaV3, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Write");

    let idea: Idea = new_idea.into_inner().into();
    let id = parse_uuid!(info.id, "idea ID");
    let cid = parse_uuid!(info.collection, "collection ID");
    let uid = parse_uuid!(token.oid(), "auth token oid");

    ensure_user_collection(&state, &token).await?;

    let role = state
        .store
        .send(
            GetRoleAssignment {
                principal_id: uid,
                collection_id: cid,
            }
            .trace(),
        )
        .await??;

    match role.role {
        Role::Owner | Role::Contributor => state
            .store
            .send(
                StoreIdea {
                    id,
                    collection: cid,
                    name: idea.name,
                    description: idea.description,
                    tags: idea.tags,
                    completed: idea.completed,
                }
                .trace(),
            )
            .await?
            .map(|idea| idea.into()),
        _ => Err(APIError::new(
            403,
            "Forbidden",
            "You do not have permission to modify an idea within this collection.",
        )),
    }
}

#[cfg(test)]
mod tests {
    use crate::api::test::*;
    use crate::models::*;

    #[actix_rt::test]
    async fn store_idea_v1() {
        test_log_init();

        let content: IdeaV1 = test_request!(PUT "/api/v1/idea/00000000000000000000000000000001", IdeaV1 {
            id: None,
            name: "Test Idea".to_string(),
            description: "This is a test idea".to_string(),
        } => OK with content);

        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
    }

    #[actix_rt::test]
    async fn store_idea_v2() {
        test_log_init();

        let content: IdeaV2 = test_request!(PUT "/api/v2/idea/00000000000000000000000000000001", IdeaV2 {
            id: None,
            name: "Test Idea".to_string(),
            description: "This is a test idea".to_string(),
            tags: Some(hashset!("test")),
            completed: None
        } => OK with content);

        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(false));
    }

    #[actix_rt::test]
    async fn store_idea_v3_new() {
        test_log_init();

        let content: IdeaV3 = test_request!(PUT "/api/v3/idea/00000000000000000000000000000001", IdeaV3 {
            id: None,
            collection: None,
            name: "Test Idea".to_string(),
            description: "This is a test idea".to_string(),
            tags: Some(hashset!("test")),
            completed: None
        } => OK with content);

        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(
            content.collection,
            Some("00000000000000000000000000000000".into())
        );
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(false));
    }

    #[actix_rt::test]
    async fn store_idea_v3_existing() {
        test_log_init();

        test_state!(
            state = [StoreIdea {
                id: 1,
                collection: 0,
                name: "Test Idea".into(),
                description: "This is a test idea".into(),
                tags: hashset!("test"),
                ..Default::default()
            }]
        );

        let content: IdeaV3 = test_request!(PUT "/api/v3/idea/00000000000000000000000000000001", IdeaV3 {
            id: None,
            collection: None,
            name: "Test Idea".to_string(),
            description: "This is a test idea with an updated description".to_string(),
            tags: Some(hashset!("test")),
            completed: Some(true)
        } => OK with content | state = state);

        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(
            content.collection,
            Some("00000000000000000000000000000000".into())
        );
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(
            content.description,
            "This is a test idea with an updated description".to_string()
        );
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(true));
    }

    #[actix_rt::test]
    async fn store_collection_idea_v3_new() {
        test_log_init();

        test_state!(
            state = [
                StoreCollection {
                    collection_id: 7,
                    principal_id: 0,
                    name: "Test Collection".into(),
                },
                StoreRoleAssignment {
                    collection_id: 7,
                    principal_id: 0,
                    role: Role::Owner,
                }
            ]
        );

        let content: IdeaV3 = test_request!(PUT "/api/v3/collection/00000000000000000000000000000007/idea/00000000000000000000000000000001", IdeaV3 {
            id: None,
            collection: None,
            name: "Test Idea".to_string(),
            description: "This is a test idea".to_string(),
            tags: Some(hashset!("test")),
            completed: None
        } => OK with content | state = state);

        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(
            content.collection,
            Some("00000000000000000000000000000007".into())
        );
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(false));
    }

    #[actix_rt::test]
    async fn store_collection_idea_v3_existing() {
        test_log_init();

        test_state!(
            state = [
                StoreCollection {
                    collection_id: 7,
                    principal_id: 0,
                    name: "Test Collection".into(),
                },
                StoreRoleAssignment {
                    collection_id: 7,
                    principal_id: 0,
                    role: Role::Owner,
                },
                StoreIdea {
                    id: 1,
                    collection: 7,
                    name: "Test Idea".into(),
                    description: "This is a test idea".into(),
                    tags: hashset!("test"),
                    ..Default::default()
                }
            ]
        );

        let content: IdeaV3 = test_request!(PUT "/api/v3/collection/00000000000000000000000000000007/idea/00000000000000000000000000000001", IdeaV3 {
            id: None,
            collection: None,
            name: "Test Idea".to_string(),
            description: "This is a test idea with an updated description".to_string(),
            tags: Some(hashset!("test")),
            completed: Some(true)
        } => OK with content | state = state);

        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(
            content.collection,
            Some("00000000000000000000000000000007".into())
        );
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(
            content.description,
            "This is a test idea with an updated description".to_string()
        );
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(true));
    }
}
