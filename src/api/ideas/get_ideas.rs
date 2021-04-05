use actix_web::{get, web};
use super::{AuthToken, APIError, ensure_user_collection};
use crate::{models::*, telemetry::TraceMessageExt};
use super::{QueryFilter, CollectionFilter};

#[instrument(err, skip(state, token), fields(otel.kind = "internal"))]
#[get("/api/v1/ideas")]
async fn get_ideas_v1(
    state: web::Data<GlobalState>,
    token: AuthToken
) -> Result<web::Json<Vec<IdeaV1>>, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Read");
    
    let uid = parse_uuid!(token.oid(), "auth token oid");

    state.store.send(GetIdeas { collection: uid, is_completed: None, tag: None }.trace()).await?.map(|ideas| web::Json(ideas.iter().map(|i| i.clone().into()).collect()))
}

#[instrument(err, skip(state, token), fields(otel.kind = "internal"))]
#[get("/api/v2/ideas")]
async fn get_ideas_v2(
    (query, state, token): (web::Query<QueryFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<web::Json<Vec<IdeaV2>>, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Read");
    
    let uid = parse_uuid!(token.oid(), "auth token oid");

    state.store.send(GetIdeas {
        collection: uid,
        is_completed: query.complete.clone(), 
        tag: query.tag.clone()
    }.trace()).await?.map(|ideas| web::Json(ideas.iter().map(|i| i.clone().into()).collect()))
}

#[instrument(err, skip(state, token), fields(otel.kind = "internal"))]
#[get("/api/v3/ideas")]
async fn get_ideas_v3(
    (query, state, token): (web::Query<QueryFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<web::Json<Vec<IdeaV3>>, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Read");
    
    let uid = parse_uuid!(token.oid(), "auth token oid");

    ensure_user_collection(&state, &token).await?;
    state.store.send(GetRoleAssignment { principal_id: uid, collection_id: uid }).await??;

    state.store.send(GetIdeas {
        collection: uid,
        is_completed: query.complete.clone(), 
        tag: query.tag.clone()
    }.trace()).await?.map(|ideas| web::Json(ideas.iter().map(|i| i.clone().into()).collect()))
}

#[instrument(err, skip(state, token), fields(otel.kind = "internal"))]
#[get("/api/v3/collection/{collection}/ideas")]
async fn get_collection_ideas_v3(
    (info, query, state, token): (web::Path<CollectionFilter>, web::Query<QueryFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<web::Json<Vec<IdeaV3>>, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Read");
    
    let cid = parse_uuid!(info.collection, "collection ID");
    let uid = parse_uuid!(token.oid(), "auth token oid");
        
    ensure_user_collection(&state, &token).await?;
    state.store.send(GetRoleAssignment { principal_id: uid, collection_id: cid }.trace()).await??;

    state.store.send(GetIdeas {
        collection: cid,
        is_completed: query.complete.clone(), 
        tag: query.tag.clone()
    }.trace()).await?.map(|ideas| web::Json(ideas.iter().map(|i| i.clone().into()).collect()))
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::api::test::*;

    #[actix_rt::test]
    async fn get_ideas_v1() {
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

        let content: Vec<IdeaV1> = test_request!(GET "/api/v1/ideas" => OK with content | state = state);
        assert_eq!(content.len(), 1);
        assert_ne!(content[0].id, None);
        assert_eq!(content[0].name, "Test Idea".to_string());
        assert_eq!(content[0].description, "This is a test idea".to_string());
    }

    #[actix_rt::test]
    async fn get_ideas_v2() {
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

        let content: Vec<IdeaV2> = test_request!(GET "/api/v2/ideas" => OK with content | state = state);
        assert!(content.len() >= 1);
        assert_ne!(content[0].id, None);
        assert_eq!(content[0].name, "Test Idea".to_string());
        assert_eq!(content[0].description, "This is a test idea".to_string());
        assert_eq!(content[0].tags, Some(hashset!("test")));
        assert_eq!(content[0].completed, Some(false));
    }

    #[actix_rt::test]
    async fn get_ideas_v3() {
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

        let content: Vec<IdeaV3> = test_request!(GET "/api/v3/ideas" => OK with content | state = state);
        assert!(content.len() >= 1);
        assert_ne!(content[0].id, None);
        assert_eq!(content[0].collection, Some("00000000000000000000000000000000".into()));
        assert_eq!(content[0].name, "Test Idea".to_string());
        assert_eq!(content[0].description, "This is a test idea".to_string());
        assert_eq!(content[0].tags, Some(hashset!("test")));
        assert_eq!(content[0].completed, Some(false));
    }

    #[actix_rt::test]
    async fn get_collection_ideas_v3() {
        test_log_init();

        test_state!(state = [
            StoreCollection {
                collection_id: 7,
                principal_id: 0,
                name: "Test Collection".into(),
                ..Default::default()
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

        let content: Vec<IdeaV3> = test_request!(GET "/api/v3/collection/00000000000000000000000000000007/ideas" => OK with content | state = state);
        assert_eq!(content.len(), 1);
        assert_eq!(content[0].id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content[0].collection, Some("00000000000000000000000000000007".into()));
        assert_eq!(content[0].name, "Test Idea".to_string());
        assert_eq!(content[0].description, "This is a test idea".to_string());
        assert_eq!(content[0].tags, Some(hashset!("test")));
        assert_eq!(content[0].completed, Some(false));
    }
}