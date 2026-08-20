use actix_web::web;
use rex_api::{ApiError, IdeaV1, IdeaV2, IdeaV3};
use tracing::instrument;

use crate::{
    auth::AuthToken,
    db::Store,
    models::{IdeaFilter, parse_id_or_400},
    services::Services,
    web::api::{CollectionFilter, QueryFilter, ensure_user_collection},
};

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn get_ideas_v1<S: Services>(
    services: web::Data<S>,
    token: AuthToken,
) -> Result<web::Json<Vec<IdeaV1>>, ApiError> {
    let uid = token.principal_id();

    services
        .store()
        .get_ideas(uid, IdeaFilter::default())
        .await
        .map(|ideas| web::Json(ideas.into_iter().map(Into::into).collect()))
}

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn get_ideas_v2<S: Services>(
    query: web::Query<QueryFilter>,
    services: web::Data<S>,
    token: AuthToken,
) -> Result<web::Json<Vec<IdeaV2>>, ApiError> {
    let uid = token.principal_id();

    services
        .store()
        .get_ideas(uid, (&*query).into())
        .await
        .map(|ideas| web::Json(ideas.into_iter().map(Into::into).collect()))
}

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn get_ideas_v3<S: Services>(
    query: web::Query<QueryFilter>,
    services: web::Data<S>,
    token: AuthToken,
) -> Result<web::Json<Vec<IdeaV3>>, ApiError> {
    let uid = token.principal_id();

    ensure_user_collection(services.get_ref(), &token).await?;
    services.store().get_role_assignment(uid, uid).await?;

    services
        .store()
        .get_ideas(uid, (&*query).into())
        .await
        .map(|ideas| web::Json(ideas.into_iter().map(Into::into).collect()))
}

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn get_collection_ideas_v3<S: Services>(
    info: web::Path<CollectionFilter>,
    query: web::Query<QueryFilter>,
    services: web::Data<S>,
    token: AuthToken,
) -> Result<web::Json<Vec<IdeaV3>>, ApiError> {
    let cid = parse_id_or_400(&info.collection, "collection ID")?;
    let uid = token.principal_id();

    ensure_user_collection(services.get_ref(), &token).await?;
    services.store().get_role_assignment(cid, uid).await?;

    services
        .store()
        .get_ideas(cid, (&*query).into())
        .await
        .map(|ideas| web::Json(ideas.into_iter().map(Into::into).collect()))
}

#[cfg(test)]
mod tests {
    use rex_api::{IdeaV1, IdeaV2, IdeaV3};

    use crate::{hashset, test_request, test_state, testing::*};

    #[actix_rt::test]
    async fn get_ideas_v1() {
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

        let content: Vec<IdeaV1> =
            test_request!(GET "/api/v1/ideas" => OK with content | state = state);
        assert_eq!(content.len(), 1);
        assert_ne!(content[0].id, None);
        assert_eq!(content[0].name, "Test Idea".to_string());
        assert_eq!(content[0].description, "This is a test idea".to_string());
    }

    #[actix_rt::test]
    async fn get_ideas_v2() {
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

        let content: Vec<IdeaV2> =
            test_request!(GET "/api/v2/ideas" => OK with content | state = state);
        assert!(!content.is_empty());
        assert_ne!(content[0].id, None);
        assert_eq!(content[0].name, "Test Idea".to_string());
        assert_eq!(content[0].description, "This is a test idea".to_string());
        assert_eq!(content[0].tags, Some(hashset!("test")));
        assert_eq!(content[0].completed, Some(false));
    }

    #[actix_rt::test]
    async fn get_ideas_v3() {
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

        let content: Vec<IdeaV3> =
            test_request!(GET "/api/v3/ideas" => OK with content | state = state);
        assert!(!content.is_empty());
        assert_ne!(content[0].id, None);
        assert_eq!(
            content[0].collection,
            Some("00000000000000000000000000000000".into())
        );
        assert_eq!(content[0].name, "Test Idea".to_string());
        assert_eq!(content[0].description, "This is a test idea".to_string());
        assert_eq!(content[0].tags, Some(hashset!("test")));
        assert_eq!(content[0].completed, Some(false));
    }

    #[actix_rt::test]
    async fn get_ideas_v2_filters_by_tag() {
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
                    name: "Tagged".into(),
                    tags: hashset!("wanted"),
                    ..Default::default()
                },
                idea {
                    id: 2,
                    collection_id: 0,
                    name: "Untagged".into(),
                    ..Default::default()
                },
            ]
        );

        let content: Vec<IdeaV2> =
            test_request!(GET "/api/v2/ideas?tag=wanted" => OK with content | state = state);

        assert_eq!(content.len(), 1);
        assert_eq!(content[0].name, "Tagged");
    }

    #[actix_rt::test]
    async fn get_collection_ideas_v3() {
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

        let content: Vec<IdeaV3> = test_request!(GET "/api/v3/collection/00000000000000000000000000000007/ideas" => OK with content | state = state);
        assert_eq!(content.len(), 1);
        assert_eq!(
            content[0].id,
            Some("00000000000000000000000000000001".into())
        );
        assert_eq!(
            content[0].collection,
            Some("00000000000000000000000000000007".into())
        );
        assert_eq!(content[0].name, "Test Idea".to_string());
        assert_eq!(content[0].description, "This is a test idea".to_string());
        assert_eq!(content[0].tags, Some(hashset!("test")));
        assert_eq!(content[0].completed, Some(false));
    }
}
