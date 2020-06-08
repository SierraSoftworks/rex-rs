use actix_web::{get, web};
use super::{AuthToken, APIError, ensure_user_collection};
use crate::models::*;
use super::{models, IdFilter, CollectionIdFilter};

#[get("/api/v1/idea/{id}")]
async fn get_idea_v1(
    (info, state, token): (web::Path<IdFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<models::IdeaV1, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Read");
    
    let id = parse_uuid!(info.id, idea ID);
    let uid = parse_uuid!(token.oid, auth token oid);
    
    state.store.send(GetIdea { collection: uid, id: id }).await?.map(|idea| idea.clone().into())
}

#[get("/api/v2/idea/{id}")]
async fn get_idea_v2(
    (info, state, token): (web::Path<IdFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<models::IdeaV2, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Read");
    
    let id = parse_uuid!(info.id, idea ID);
    let uid = parse_uuid!(token.oid, auth token oid);
        
    state.store.send(GetIdea { collection: uid, id: id }).await?.map(|idea| idea.clone().into())
}

#[get("/api/v3/idea/{id}")]
async fn get_idea_v3(
    (info, state, token): (web::Path<IdFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<models::IdeaV3, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Read");
    
    let id = parse_uuid!(info.id, idea ID);
    let uid = parse_uuid!(token.oid, auth token oid);
        
    ensure_user_collection(&state, &token).await?;
    
    state.store.send(GetIdea { collection: uid, id: id }).await?.map(|idea| idea.clone().into())
}

#[get("/api/v3/collection/{collection}/idea/{id}")]
async fn get_collection_idea_v3(
    (info, state, token): (web::Path<CollectionIdFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<models::IdeaV3, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Read");
    
    let id = parse_uuid!(info.id, idea ID);
    let cid = parse_uuid!(info.collection, collection ID);
    let uid = parse_uuid!(token.oid, auth token oid);
        
    ensure_user_collection(&state, &token).await?;

    state.store.send(GetRoleAssignment { principal_id: uid, collection_id: cid }).await??;

    state.store.send(GetIdea { collection: cid, id: id }).await?.map(|idea| idea.clone().into())
}

#[cfg(test)]
mod tests {
    use super::models::*;
    use crate::models::*;
    use actix_web::test;
    use http::{Method, StatusCode};
    use crate::api::test::*;

    #[actix_rt::test]
    async fn get_idea_v1() {
        test_log_init();

        let state = GlobalState::new();
        state.store.send(StoreIdea {
            id: 1,
            collection: 0,
            name: "Test Idea".into(),
            description: "This is a test idea".into(),
            tags: hashset!("test"),
            ..Default::default()
        }).await.expect("the actor should run").expect("the idea should be stored");

        let mut app = get_test_app(state.clone()).await;

        let req = test::TestRequest::with_uri("/api/v1/idea/00000000000000000000000000000001")
            .method(Method::GET)
            .header("Authorization", auth_token()).to_request();

        let mut response = test::call_service(&mut app, req).await;
        assert_status(&mut response, StatusCode::OK).await;
        
        let content: IdeaV1 = get_content(&mut response).await;
        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
    }

    #[actix_rt::test]
    async fn get_idea_v2() {
        test_log_init();

        let state = GlobalState::new();
        state.store.send(StoreIdea {
            id: 1,
            collection: 0,
            name: "Test Idea".into(),
            description: "This is a test idea".into(),
            tags: hashset!("test"),
            ..Default::default()
        }).await.expect("the actor should run").expect("the idea should be stored");

        let mut app = get_test_app(state.clone()).await;

        let req = test::TestRequest::with_uri("/api/v2/idea/00000000000000000000000000000001")
            .method(Method::GET)
            .header("Authorization", auth_token())
            .to_request();

        let mut response = test::call_service(&mut app, req).await;
        assert_status(&mut response, StatusCode::OK).await;
        
        let content: IdeaV2 = get_content(&mut response).await;
        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(false));
    }

    #[actix_rt::test]
    async fn get_idea_v3() {
        test_log_init();

        let state = GlobalState::new();
        state.store.send(StoreIdea {
            id: 1,
            collection: 0,
            name: "Test Idea".into(),
            description: "This is a test idea".into(),
            tags: hashset!("test"),
            ..Default::default()
        }).await.expect("the actor should run").expect("the idea should be stored");

        let mut app = get_test_app(state.clone()).await;

        let req = test::TestRequest::with_uri("/api/v3/idea/00000000000000000000000000000001")
            .method(Method::GET)
            .header("Authorization", auth_token())
            .to_request();

        let mut response = test::call_service(&mut app, req).await;
        assert_status(&mut response, StatusCode::OK).await;
        
        let content: IdeaV3 = get_content(&mut response).await;
        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.collection, Some("00000000000000000000000000000000".into()));
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(false));
    }

    #[actix_rt::test]
    async fn get_collection_idea_v3() {
        test_log_init();

        let state = GlobalState::new();
        state.store.send(StoreCollection {
            collection_id: 7,
            principal_id: 0,
            name: "Test Collection".into(),
            ..Default::default()
        }).await.expect("the actor should run").expect("the collection should be stored");

        state.store.send(StoreRoleAssignment {
            collection_id: 7,
            principal_id: 0,
            role: Role::Owner,
        }).await.expect("the actor should run").expect("the role assignment should be stored");
        state.store.send(StoreIdea {
            id: 1,
            collection: 7,
            name: "Test Idea".into(),
            description: "This is a test idea".into(),
            tags: hashset!("test"),
            ..Default::default()
        }).await.expect("the actor should run").expect("the idea should be stored");

        let mut app = get_test_app(state.clone()).await;

        let req = test::TestRequest::with_uri("/api/v3/collection/00000000000000000000000000000007/idea/00000000000000000000000000000001")
            .method(Method::GET)
            .header("Authorization", auth_token())
            .to_request();

        let mut response = test::call_service(&mut app, req).await;
        assert_status(&mut response, StatusCode::OK).await;
        
        let content: IdeaV3 = get_content(&mut response).await;
        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.collection, Some("00000000000000000000000000000007".into()));
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(false));
    }
}