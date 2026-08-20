use actix_web::web;
use rex_api::{ApiError, IdeaV1, IdeaV2, IdeaV3};
use tracing::instrument;

use crate::{
    auth::AuthToken,
    db::Store,
    models::{Idea, parse_id_or_400},
    services::Services,
    web::{
        ApiResponse,
        api::{CollectionIdFilter, IdFilter, ensure_user_collection},
    },
};

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn store_idea_v1<S: Services>(
    info: web::Path<IdFilter>,
    new_idea: web::Json<IdeaV1>,
    services: web::Data<S>,
    token: AuthToken,
) -> Result<ApiResponse<IdeaV1>, ApiError> {
    let idea: Idea = new_idea.into_inner().into();
    let id = parse_id_or_400(&info.id, "idea ID")?;
    let uid = token.principal_id();

    ensure_user_collection(services.get_ref(), &token).await?;

    services
        .store()
        .store_idea(Idea {
            id,
            collection_id: uid,
            ..idea
        })
        .await
        .map(|idea| ApiResponse(idea.into()))
}

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn store_idea_v2<S: Services>(
    info: web::Path<IdFilter>,
    new_idea: web::Json<IdeaV2>,
    services: web::Data<S>,
    token: AuthToken,
) -> Result<ApiResponse<IdeaV2>, ApiError> {
    let idea: Idea = new_idea.into_inner().into();
    let id = parse_id_or_400(&info.id, "idea ID")?;
    let uid = token.principal_id();

    ensure_user_collection(services.get_ref(), &token).await?;

    services
        .store()
        .store_idea(Idea {
            id,
            collection_id: uid,
            ..idea
        })
        .await
        .map(|idea| ApiResponse(idea.into()))
}

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn store_idea_v3<S: Services>(
    info: web::Path<IdFilter>,
    new_idea: web::Json<IdeaV3>,
    services: web::Data<S>,
    token: AuthToken,
) -> Result<ApiResponse<IdeaV3>, ApiError> {
    let idea: Idea = new_idea.into_inner().into();
    let id = parse_id_or_400(&info.id, "idea ID")?;
    let uid = token.principal_id();

    ensure_user_collection(services.get_ref(), &token).await?;

    services
        .store()
        .store_idea(Idea {
            id,
            collection_id: uid,
            ..idea
        })
        .await
        .map(|idea| ApiResponse(idea.into()))
}

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn store_collection_idea_v3<S: Services>(
    info: web::Path<CollectionIdFilter>,
    new_idea: web::Json<IdeaV3>,
    services: web::Data<S>,
    token: AuthToken,
) -> Result<ApiResponse<IdeaV3>, ApiError> {
    let idea: Idea = new_idea.into_inner().into();
    let id = parse_id_or_400(&info.id, "idea ID")?;
    let cid = parse_id_or_400(&info.collection, "collection ID")?;
    let uid = token.principal_id();

    ensure_user_collection(services.get_ref(), &token).await?;

    let role = services.store().get_role_assignment(cid, uid).await?;

    if !role.role.can_write() {
        return Err(ApiError::forbidden(
            "You do not have permission to modify an idea within this collection.",
        ));
    }

    services
        .store()
        .store_idea(Idea {
            id,
            collection_id: cid,
            ..idea
        })
        .await
        .map(|idea| ApiResponse(idea.into()))
}

#[cfg(test)]
mod tests {
    use rex_api::{IdeaV1, IdeaV2, IdeaV3};

    use crate::{hashset, test_request, test_state, testing::*};

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
            state = [
                collection {
                    collection_id: 0,
                    user_id: 0,
                    name: "My Ideas".into()
                },
                idea {
                    id: 1,
                    collection_id: 0,
                    name: "Test Idea".into(),
                    description: "This is a test idea".into(),
                    tags: hashset!("test"),
                    ..Default::default()
                },
            ]
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
            state = [collection {
                collection_id: 7,
                user_id: 0,
                name: "Test Collection".into()
            },]
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
                collection {
                    collection_id: 7,
                    user_id: 0,
                    name: "Test Collection".into()
                },
                idea {
                    id: 1,
                    collection_id: 7,
                    name: "Test Idea".into(),
                    description: "This is a test idea".into(),
                    tags: hashset!("test"),
                    ..Default::default()
                },
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
