use actix_web::{get, web};
use super::{AuthToken, APIError, ensure_user_collection};
use crate::{models::*, telemetry::TraceMessageExt};
use super::{CollectionFilter, QueryFilter};

#[instrument(err, skip(state, token), fields(otel.kind = "internal"))]
#[get("/api/v1/idea/random")]
async fn get_random_idea_v1(state: web::Data<GlobalState>, token: AuthToken) -> Result<IdeaV1, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Read");
    
    let uid = parse_uuid!(token.oid(), "auth token oid");

    state.store.send(GetRandomIdea { collection: uid, is_completed: None, tag: None }.trace()).await?.map(|idea| idea.into())
}

#[instrument(err, skip(state, token), fields(otel.kind = "internal"))]
#[get("/api/v2/idea/random")]
async fn get_random_idea_v2(
    (query, state, token): (web::Query<QueryFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<IdeaV2, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Read");
    
    let uid = parse_uuid!(token.oid(), "auth token oid");

    state.store.send(GetRandomIdea { collection: uid, is_completed: query.complete, tag: query.tag.clone() }.trace()).await?.map(|idea| idea.into())
}

#[instrument(err, skip(state, token), fields(otel.kind = "internal"))]
#[get("/api/v3/idea/random")]
async fn get_random_idea_v3(
    (query, state, token): (web::Query<QueryFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<IdeaV3, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Read");
    
    let uid = parse_uuid!(token.oid(), "auth token oid");

    ensure_user_collection(&state, &token).await?;
    state.store.send(GetRoleAssignment { principal_id: uid, collection_id: uid }.trace()).await??;

    state.store.send(GetRandomIdea { collection: uid, is_completed: query.complete, tag: query.tag.clone() }.trace()).await?.map(|idea| idea.into())
}

#[instrument(err, skip(state, token), fields(otel.kind = "internal"))]
#[get("/api/v3/collection/{collection}/idea/random")]
async fn get_random_collection_idea_v3(
    (info, query, state, token): (web::Path<CollectionFilter>, web::Query<QueryFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<IdeaV3, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Read");
    
    let cid = parse_uuid!(info.collection, "collection ID");
    let uid = parse_uuid!(token.oid(), "auth token oid");
        
    state.store.send(GetRoleAssignment { principal_id: uid, collection_id: cid }.trace()).await??;

    state.store.send(GetRandomIdea { collection: cid, is_completed: query.complete, tag: query.tag.clone() }.trace()).await?.map(|idea| idea.into())
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::api::test::*;

    #[actix_rt::test]
    async fn random_idea_v1() {
        test_log_init();

        test_state!(state = [
            StoreIdea {
                id: 1,
                collection: 0,
                name: "Test Idea".into(),
                description: "This is a test idea".into(),
                tags: hashset!("test"),
                ..Default::default()
            }
        ]);

        
        let content: IdeaV1 = test_request!(GET "/api/v1/idea/random" => OK with content | state = state);
        assert_ne!(content.id, None);
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
    }

    #[actix_rt::test]
    async fn random_idea_v2() {
        test_log_init();

        test_state!(state = [
            StoreIdea {
                id: 1,
                collection: 0,
                name: "Test Idea".into(),
                description: "This is a test idea".into(),
                tags: hashset!("test"),
                ..Default::default()
            }
        ]);
        
        let content: IdeaV2 = test_request!(GET "/api/v2/idea/random" => OK with content | state = state);
        assert_ne!(content.id, None);
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(false));
    }

    #[actix_rt::test]
    async fn random_idea_v3() {
        test_log_init();

        test_state!(state = [
            StoreIdea {
                id: 1,
                collection: 0,
                name: "Test Idea".into(),
                description: "This is a test idea".into(),
                tags: hashset!("test"),
                ..Default::default()
            }
        ]);

        let content: IdeaV3 = test_request!(GET "/api/v3/idea/random" => OK with content | state = state);
        assert_ne!(content.id, None);
        assert_eq!(content.collection, Some("00000000000000000000000000000000".into()));
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(false));
    }

    #[actix_rt::test]
    async fn random_collection_idea_v3() {
        test_log_init();

        test_state!(state = [
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
        ]);

        let content: IdeaV3 = test_request!(GET "/api/v3/collection/00000000000000000000000000000007/idea/random" => OK with content | state = state);
        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.collection, Some("00000000000000000000000000000007".into()));
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(false));
    }
}