use actix_web::{get, web};
use super::{AuthToken, APIError, ensure_user_collection};
use crate::models::*;
use super::{models, QueryFilter, CollectionFilter};

#[get("/api/v1/ideas")]
async fn get_ideas_v1(
    state: web::Data<GlobalState>,
    token: AuthToken
) -> Result<web::Json<Vec<models::IdeaV1>>, APIError> {
    let oid = u128::from_str_radix(token.oid.replace("-", "").as_str(), 16)
        .or(Err(APIError::new(400, "Bad Request", "The auth token OID you provided could not be parsed. Please check it and try again.")))?;

    state.store.send(GetIdeas { collection: oid, is_completed: None, tag: None }).await?.map(|ideas| web::Json(ideas.iter().map(|i| i.clone().into()).collect()))
}

#[get("/api/v2/ideas")]
async fn get_ideas_v2(
    (query, state, token): (web::Query<QueryFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<web::Json<Vec<models::IdeaV2>>, APIError> {
    let oid = u128::from_str_radix(token.oid.replace("-", "").as_str(), 16)
        .or(Err(APIError::new(400, "Bad Request", "The auth token OID you provided could not be parsed. Please check it and try again.")))?;

    state.store.send(GetIdeas {
        collection: oid,
        is_completed: query.complete.clone(), 
        tag: query.tag.clone()
    }).await?.map(|ideas| web::Json(ideas.iter().map(|i| i.clone().into()).collect()))
}

#[get("/api/v3/collection/{collection}/ideas")]
async fn get_ideas_v3(
    (path, query, state, token): (web::Path<CollectionFilter>, web::Query<QueryFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<web::Json<Vec<models::IdeaV3>>, APIError> {
    let cid = u128::from_str_radix(&path.collection, 16)
        .or(Err(APIError::new(400, "Bad Request", "The collection ID you provided could not be parsed. Please check it and try again.")))?;
    
    let uid = u128::from_str_radix(token.oid.replace("-", "").as_str(), 16)
        .or(Err(APIError::new(400, "Bad Request", "The auth token OID you provided could not be parsed. Please check it and try again.")))?;
        
    ensure_user_collection(&state, &token).await?;

    state.store.send(GetRoleAssignment { principal_id: uid, collection_id: cid }).await??;

    state.store.send(GetIdeas {
        collection: cid,
        is_completed: query.complete.clone(), 
        tag: query.tag.clone()
    }).await?.map(|ideas| web::Json(ideas.iter().map(|i| i.clone().into()).collect()))
}

#[cfg(test)]
mod tests {
    use super::models::*;
    use crate::models::*;
    use actix_web::test;
    use http::{Method, StatusCode};
    use crate::api::test::*;

    #[actix_rt::test]
    async fn get_ideas_v1() {
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

        let req = test::TestRequest::with_uri("/api/v1/ideas")
            .method(Method::GET)
            .header("Authorization", auth_token()).to_request();

        let mut response = test::call_service(&mut app, req).await;
        assert_status(&mut response, StatusCode::OK).await;
        
        let content: Vec<IdeaV1> = get_content(&mut response).await;
        assert_eq!(content.len(), 1);
        assert_ne!(content[0].id, None);
        assert_eq!(content[0].name, "Test Idea".to_string());
        assert_eq!(content[0].description, "This is a test idea".to_string());
    }

    #[actix_rt::test]
    async fn get_ideas_v2() {
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

        let req = test::TestRequest::with_uri("/api/v2/ideas")
            .method(Method::GET)
            .header("Authorization", auth_token())
            .to_request();

        let mut response = test::call_service(&mut app, req).await;
        assert_status(&mut response, StatusCode::OK).await;
        
        let content: Vec<IdeaV2> = get_content(&mut response).await;
        assert_eq!(content.len(), 1);
        assert_ne!(content[0].id, None);
        assert_eq!(content[0].name, "Test Idea".to_string());
        assert_eq!(content[0].description, "This is a test idea".to_string());
        assert_eq!(content[0].tags, Some(hashset!("test")));
        assert_eq!(content[0].completed, Some(false));
    }

    #[actix_rt::test]
    async fn get_ideas_v3() {
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

        let req = test::TestRequest::with_uri("/api/v3/collection/00000000000000000000000000000007/ideas")
            .method(Method::GET)
            .header("Authorization", auth_token())
            .to_request();

        let mut response = test::call_service(&mut app, req).await;
        assert_status(&mut response, StatusCode::OK).await;
        
        let content: Vec<IdeaV3> = get_content(&mut response).await;
        assert_eq!(content.len(), 1);
        assert_eq!(content[0].id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content[0].collection, Some("00000000000000000000000000000007".into()));
        assert_eq!(content[0].name, "Test Idea".to_string());
        assert_eq!(content[0].description, "This is a test idea".to_string());
        assert_eq!(content[0].tags, Some(hashset!("test")));
        assert_eq!(content[0].completed, Some(false));
    }
}