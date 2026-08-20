use actix_web::web;
use rex_api::{ApiError, IdeaV1, IdeaV2, IdeaV3};
use tracing::instrument;

use crate::{
    auth::AuthToken,
    db::Store,
    models::parse_id_or_400,
    services::Services,
    web::{
        ApiResponse,
        api::{CollectionIdFilter, IdFilter, ensure_user_collection},
    },
};

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn get_idea_v1<S: Services>(
    info: web::Path<IdFilter>,
    services: web::Data<S>,
    token: AuthToken,
) -> Result<ApiResponse<IdeaV1>, ApiError> {
    let id = parse_id_or_400(&info.id, "idea ID")?;
    let uid = token.principal_id();

    services
        .store()
        .get_idea(uid, id)
        .await
        .map(|idea| ApiResponse(idea.into()))
}

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn get_idea_v2<S: Services>(
    info: web::Path<IdFilter>,
    services: web::Data<S>,
    token: AuthToken,
) -> Result<ApiResponse<IdeaV2>, ApiError> {
    let id = parse_id_or_400(&info.id, "idea ID")?;
    let uid = token.principal_id();

    services
        .store()
        .get_idea(uid, id)
        .await
        .map(|idea| ApiResponse(idea.into()))
}

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn get_idea_v3<S: Services>(
    info: web::Path<IdFilter>,
    services: web::Data<S>,
    token: AuthToken,
) -> Result<ApiResponse<IdeaV3>, ApiError> {
    let id = parse_id_or_400(&info.id, "idea ID")?;
    let uid = token.principal_id();

    ensure_user_collection(services.get_ref(), &token).await?;

    services
        .store()
        .get_idea(uid, id)
        .await
        .map(|idea| ApiResponse(idea.into()))
}

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn get_collection_idea_v3<S: Services>(
    info: web::Path<CollectionIdFilter>,
    services: web::Data<S>,
    token: AuthToken,
) -> Result<ApiResponse<IdeaV3>, ApiError> {
    let id = parse_id_or_400(&info.id, "idea ID")?;
    let cid = parse_id_or_400(&info.collection, "collection ID")?;
    let uid = token.principal_id();

    ensure_user_collection(services.get_ref(), &token).await?;

    // Any role at all is enough to read; the absence of one is a 403.
    services.store().get_role_assignment(cid, uid).await?;

    services
        .store()
        .get_idea(cid, id)
        .await
        .map(|idea| ApiResponse(idea.into()))
}

#[cfg(test)]
mod tests {
    use rex_api::{IdeaV1, IdeaV2, IdeaV3};

    use crate::{db::Store, hashset, models::*, test_request, test_state, testing::*};

    #[actix_rt::test]
    async fn get_idea_v1() {
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

        let content: IdeaV1 = test_request!(GET "/api/v1/idea/00000000000000000000000000000001" => OK with content | state = state);
        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
    }

    #[actix_rt::test]
    async fn get_idea_v2() {
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

        let content: IdeaV2 = test_request!(GET "/api/v2/idea/00000000000000000000000000000001" => OK with content | state = state);
        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(false));
    }

    #[actix_rt::test]
    async fn get_idea_v3() {
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

        let content: IdeaV3 = test_request!(GET "/api/v3/idea/00000000000000000000000000000001" => OK with content | state = state);
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
    async fn get_collection_idea_v3() {
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

        let content: IdeaV3 = test_request!(GET "/api/v3/collection/00000000000000000000000000000007/idea/00000000000000000000000000000001" => OK with content | state = state);
        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(
            content.collection,
            Some("00000000000000000000000000000007".into())
        );
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(false));

        // Touching any v3 endpoint should have registered the caller, so that
        // somebody else can find them by email hash to share a collection.
        let user = state
            .store()
            .get_user(email_hash("developer@localhost"))
            .await
            .expect("the user should have been registered");

        assert_eq!(user.first_name, "Local");
        assert_eq!(user.principal_id, 0);
    }

    #[actix_rt::test]
    async fn get_collection_idea_v3_without_a_role() {
        test_log_init();

        test_state!(
            state = [
                collection {
                    collection_id: 7,
                    user_id: 9,
                    name: "Somebody Else's".into()
                },
                idea {
                    id: 1,
                    collection_id: 7,
                    ..Default::default()
                },
            ]
        );

        test_request!(GET "/api/v3/collection/00000000000000000000000000000007/idea/00000000000000000000000000000001" => FORBIDDEN | state = state);
    }
}
