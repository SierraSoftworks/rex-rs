use actix_web::web;
use rex_api::{ApiError, IdeaV1, IdeaV2, IdeaV3};
use tracing::instrument;

use crate::{
    auth::AuthToken,
    db::Store,
    models::{Idea, new_id, parse_id_or_400},
    services::Services,
    web::{
        ApiResponse,
        api::{CollectionFilter, ensure_user_collection},
    },
};

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn new_idea_v1<S: Services>(
    new_idea: web::Json<IdeaV1>,
    services: web::Data<S>,
    token: AuthToken,
) -> Result<ApiResponse<IdeaV1>, ApiError> {
    let idea: Idea = new_idea.into_inner().into();
    let uid = token.principal_id();

    ensure_user_collection(services.get_ref(), &token).await?;

    services
        .store()
        .store_idea(Idea {
            id: new_id(),
            collection_id: uid,
            completed: false,
            ..idea
        })
        .await
        .map(|idea| ApiResponse(idea.into()))
}

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn new_idea_v2<S: Services>(
    new_idea: web::Json<IdeaV2>,
    services: web::Data<S>,
    token: AuthToken,
) -> Result<ApiResponse<IdeaV2>, ApiError> {
    let idea: Idea = new_idea.into_inner().into();
    let uid = token.principal_id();

    ensure_user_collection(services.get_ref(), &token).await?;

    services
        .store()
        .store_idea(Idea {
            id: new_id(),
            collection_id: uid,
            completed: false,
            ..idea
        })
        .await
        .map(|idea| ApiResponse(idea.into()))
}

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn new_idea_v3<S: Services>(
    new_idea: web::Json<IdeaV3>,
    services: web::Data<S>,
    token: AuthToken,
) -> Result<ApiResponse<IdeaV3>, ApiError> {
    let idea: Idea = new_idea.into_inner().into();
    let uid = token.principal_id();

    ensure_user_collection(services.get_ref(), &token).await?;

    services
        .store()
        .store_idea(Idea {
            id: new_id(),
            collection_id: uid,
            completed: false,
            ..idea
        })
        .await
        .map(|idea| ApiResponse(idea.into()))
}

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn new_collection_idea_v3<S: Services>(
    new_idea: web::Json<IdeaV3>,
    info: web::Path<CollectionFilter>,
    services: web::Data<S>,
    token: AuthToken,
) -> Result<ApiResponse<IdeaV3>, ApiError> {
    let idea: Idea = new_idea.into_inner().into();
    let cid = parse_id_or_400(&info.collection, "collection ID")?;
    let uid = token.principal_id();

    if cid == uid {
        ensure_user_collection(services.get_ref(), &token).await?;
    }

    let role = services.store().get_role_assignment(cid, uid).await?;

    if !role.role.can_write() {
        return Err(ApiError::forbidden(
            "You do not have permission to add an idea to this collection.",
        ));
    }

    services
        .store()
        .store_idea(Idea {
            id: new_id(),
            collection_id: cid,
            ..idea
        })
        .await
        .map(|idea| ApiResponse(idea.into()))
}

#[cfg(test)]
mod tests {
    use rex_api::{IdeaV1, IdeaV2, IdeaV3};

    use crate::{db::Store, hashset, models::parse_id, test_request, test_state, testing::*};

    #[actix_rt::test]
    async fn new_idea_v1() {
        test_log_init();

        test_state!(state = []);

        let content: IdeaV1 = test_request!(POST "/api/v1/ideas", IdeaV1 {
            id: None,
            name: "Test Idea".to_string(),
            description: "This is a test idea".to_string(),
        } => CREATED with location =~ "/api/v1/idea/", content | state = state);

        assert_ne!(content.id, None);
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());

        state
            .store()
            .get_idea(0, parse_id(&content.id.unwrap()).unwrap())
            .await
            .expect("The idea should exist in the store");
    }

    #[actix_rt::test]
    async fn new_idea_v2() {
        test_log_init();

        test_state!(state = []);

        let content: IdeaV2 = test_request!(POST "/api/v2/ideas", IdeaV2 {
            id: None,
            name: "Test Idea".to_string(),
            description: "This is a test idea".to_string(),
            tags: Some(hashset!("test")),
            completed: None
        } => CREATED with location =~ "/api/v2/idea/", content | state = state);

        assert_ne!(content.id, None);
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(false));

        state
            .store()
            .get_idea(0, parse_id(&content.id.unwrap()).unwrap())
            .await
            .expect("The idea should exist in the store");
    }

    #[actix_rt::test]
    async fn new_idea_v3() {
        test_log_init();

        test_state!(state = []);

        let content: IdeaV3 = test_request!(POST "/api/v3/ideas", IdeaV3 {
            id: None,
            collection: None,
            name: "Test Idea".to_string(),
            description: "This is a test idea".to_string(),
            tags: Some(hashset!("test")),
            completed: None
        } => CREATED with location =~ "/api/v3/idea/", content | state = state);

        assert_ne!(content.id, None);
        assert_eq!(
            content.collection,
            Some("00000000000000000000000000000000".into())
        );
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(false));

        state
            .store()
            .get_idea(0, parse_id(&content.id.unwrap()).unwrap())
            .await
            .expect("The idea should exist in the store");
    }

    #[actix_rt::test]
    async fn new_collection_idea_v3() {
        test_log_init();

        test_state!(
            state = [collection {
                collection_id: 7,
                user_id: 0,
                name: "Test Collection".into()
            },]
        );

        let content: IdeaV3 = test_request!(POST "/api/v3/collection/00000000000000000000000000000007/ideas", IdeaV3 {
            id: None,
            collection: None,
            name: "Test Idea".to_string(),
            description: "This is a test idea".to_string(),
            tags: Some(hashset!("test")),
            completed: None
        } => CREATED with location =~ "/api/v3/collection/00000000000000000000000000000007/idea/", content | state = state);

        assert_ne!(content.id, None);
        assert_eq!(
            content.collection,
            Some("00000000000000000000000000000007".into())
        );
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(false));

        state
            .store()
            .get_idea(7, parse_id(&content.id.unwrap()).unwrap())
            .await
            .expect("The idea should exist in the store");
    }

    #[actix_rt::test]
    async fn new_collection_idea_v3_as_a_viewer() {
        test_log_init();

        test_state!(
            state = [
                collection {
                    collection_id: 7,
                    user_id: 9,
                    name: "Somebody Else's".into()
                },
                role {
                    collection_id: 7,
                    user_id: 0,
                    role: crate::models::Role::Viewer
                },
            ]
        );

        test_request!(POST "/api/v3/collection/00000000000000000000000000000007/ideas", IdeaV3 {
            name: "Test Idea".to_string(),
            description: "This is a test idea".to_string(),
            ..Default::default()
        } => FORBIDDEN | state = state);
    }
}
