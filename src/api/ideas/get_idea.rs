use actix_web::{get, web};
use super::{AuthToken, APIError, ensure_user_collection};
use crate::models::*;
use super::{models, IdFilter, CollectionIdFilter};

#[get("/api/v1/idea/{id}")]
async fn get_idea_v1(
    (info, state, token): (web::Path<IdFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<models::IdeaV1, APIError> {
    let id = u128::from_str_radix(&info.id, 16)
        .or(Err(APIError::new(400, "Bad Request", "The idea ID you provided could not be parsed. Please check it and try again.")))?;

    let uid = u128::from_str_radix(token.oid.replace("-", "").as_str(), 16)
        .or(Err(APIError::new(400, "Bad Request", "The auth token OID you provided could not be parsed. Please check it and try again.")))?;
    
    state.store.send(GetIdea { collection: uid, id: id }).await?.map(|idea| idea.clone().into())
}

#[get("/api/v2/idea/{id}")]
async fn get_idea_v2(
    (info, state, token): (web::Path<IdFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<models::IdeaV2, APIError> {
    let id = u128::from_str_radix(&info.id, 16)
        .or(Err(APIError::new(400, "Bad Request", "The idea ID you provided could not be parsed. Please check it and try again.")))?;

    let uid = u128::from_str_radix(token.oid.replace("-", "").as_str(), 16)
        .or(Err(APIError::new(400, "Bad Request", "The auth token OID you provided could not be parsed. Please check it and try again.")))?;
        
    state.store.send(GetIdea { collection: uid, id: id }).await?.map(|idea| idea.clone().into())
}

#[get("/api/v3/idea/{id}")]
async fn get_idea_v3(
    (info, state, token): (web::Path<IdFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<models::IdeaV3, APIError> {
    let id = u128::from_str_radix(&info.id, 16)
        .or(Err(APIError::new(400, "Bad Request", "The idea ID you provided could not be parsed. Please check it and try again.")))?;

    let uid = u128::from_str_radix(token.oid.replace("-", "").as_str(), 16)
        .or(Err(APIError::new(400, "Bad Request", "The auth token OID you provided could not be parsed. Please check it and try again.")))?;
        
    ensure_user_collection(&state, &token).await?;
    
    state.store.send(GetIdea { collection: uid, id: id }).await?.map(|idea| idea.clone().into())
}

#[get("/api/v3/collection/{collection}/idea/{id}")]
async fn get_collection_idea_v3(
    (info, state, token): (web::Path<CollectionIdFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<models::IdeaV3, APIError> {
    let id = u128::from_str_radix(&info.id, 16)
        .or(Err(APIError::new(400, "Bad Request", "The idea ID you provided could not be parsed. Please check it and try again.")))?;

    let cid = u128::from_str_radix(&info.collection, 16)
        .or(Err(APIError::new(400, "Bad Request", "The collection ID you provided could not be parsed. Please check it and try again.")))?;
    
    let uid = u128::from_str_radix(token.oid.replace("-", "").as_str(), 16)
        .or(Err(APIError::new(400, "Bad Request", "The auth token OID you provided could not be parsed. Please check it and try again.")))?;
        
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