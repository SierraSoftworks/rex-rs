use actix_web::{post, web};
use super::{AuthToken, APIError, ensure_user_collection};
use crate::models::*;
use super::{models, CollectionFilter};

#[post("/api/v1/ideas")]
async fn new_idea_v1(
    (new_idea, state, token): (web::Json<models::IdeaV1>, web::Data<GlobalState>, AuthToken),
) -> Result<models::IdeaV1, APIError> {
    let idea: Idea = new_idea.into_inner().into();

    let oid = u128::from_str_radix(token.oid.replace("-", "").as_str(), 16)
        .or(Err(APIError::new(400, "Bad Request", "The auth token OID you provided could not be parsed. Please check it and try again.")))?;

    ensure_user_collection(&state, &token).await?;

    state.store.send(StoreIdea {
        id: new_id(),
        collection: oid,
        name: idea.name,
        description: idea.description,
        tags: idea.tags,
        completed: false,
    }).await?.map(|idea| idea.clone().into())
}

#[post("/api/v2/ideas")]
async fn new_idea_v2(
    (new_idea, state, token): (web::Json<models::IdeaV2>, web::Data<GlobalState>, AuthToken),
) -> Result<models::IdeaV2, APIError> {
    let idea: Idea = new_idea.into_inner().into();
    
    let oid = u128::from_str_radix(token.oid.replace("-", "").as_str(), 16)
        .or(Err(APIError::new(400, "Bad Request", "The auth token OID you provided could not be parsed. Please check it and try again.")))?;

    ensure_user_collection(&state, &token).await?;

    state.store.send(StoreIdea {
        id: new_id(),
        collection: oid,
        name: idea.name,
        description: idea.description,
        tags: idea.tags,
        completed: false,
    }).await?.map(|idea| idea.clone().into())
}

#[post("/api/v3/ideas")]
async fn new_idea_v3(
    (new_idea, state, token): (web::Json<models::IdeaV3>, web::Data<GlobalState>, AuthToken),
) -> Result<models::IdeaV3, APIError> {
    let idea: Idea = new_idea.into_inner().into();
    
    let oid = u128::from_str_radix(token.oid.replace("-", "").as_str(), 16)
        .or(Err(APIError::new(400, "Bad Request", "The auth token OID you provided could not be parsed. Please check it and try again.")))?;

    ensure_user_collection(&state, &token).await?;

    state.store.send(StoreIdea {
        id: new_id(),
        collection: oid,
        name: idea.name,
        description: idea.description,
        tags: idea.tags,
        completed: false,
    }).await?.map(|idea| idea.clone().into())
}

#[post("/api/v3/collection/{collection}/ideas")]
async fn new_collection_idea_v3(
    (new_idea, path, state, token): (web::Json<models::IdeaV3>, web::Path<CollectionFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<models::IdeaV3, APIError> {
    let idea: Idea = new_idea.into_inner().into();
    
    let cid = u128::from_str_radix(&path.collection, 16)
        .or(Err(APIError::new(400, "Bad Request", "The collection ID you provided could not be parsed. Please check it and try again.")))?;

    let uid = u128::from_str_radix(token.oid.replace("-", "").as_str(), 16)
        .or(Err(APIError::new(400, "Bad Request", "The auth token OID you provided could not be parsed. Please check it and try again.")))?;
        
    if cid == uid {
        ensure_user_collection(&state, &token).await?;
    }

    let role = state.store.send(GetRoleAssignment { principal_id: uid, collection_id: cid }).await??;

    match role.role {
        Role::Owner | Role::Contributor => {
            state.store.send(StoreIdea {
                id: new_id(),
                collection: cid,
                name: idea.name,
                description: idea.description,
                tags: idea.tags,
                completed: idea.completed,
            }).await?.map(|idea| idea.clone().into())
        },
        _ => Err(APIError::new(403, "Forbidden", "You do not have permission to add an idea to this collection."))
    }
}

#[cfg(test)]
mod tests {
    use crate::api::ideas::models::*;
    use crate::models::*;
    use actix_web::test;
    use http::{Method, StatusCode};
    use crate::api::test::*;

    #[actix_rt::test]
    async fn new_idea_v1() {
        test_log_init();

        let state = GlobalState::new();
        let mut app = get_test_app(state.clone()).await;

        let req = test::TestRequest::with_uri("/api/v1/ideas")
            .method(Method::POST)
            .header("Authorization", auth_token())
            .set_json(&IdeaV1 {
                id: None,
                name: "Test Idea".to_string(),
                description: "This is a test idea".to_string(),
            }).to_request();

        let mut response = test::call_service(&mut app, req).await;
        assert_status(&mut response, StatusCode::CREATED).await;
        assert_location_header(response.headers(), "/api/v1/idea/");

        let content: IdeaV1 = get_content(&mut response).await;
        assert_ne!(content.id, None);
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());

        state.store.send(GetIdea {
            collection: 0,
            id: u128::from_str_radix(content.id.unwrap().as_str(), 16).unwrap(),
        }).await.expect("the actor should have run").expect("The idea should exist in the store");
    }

    #[actix_rt::test]
    async fn new_idea_v2() {
        test_log_init();

        let state = GlobalState::new();
        let mut app = get_test_app(state.clone()).await;

        let req = test::TestRequest::with_uri("/api/v2/ideas")
            .method(Method::POST)
            .header("Authorization", auth_token())
            .set_json(&IdeaV2 {
                id: None,
                name: "Test Idea".to_string(),
                description: "This is a test idea".to_string(),
                tags: Some(hashset!("test")),
                completed: None
            })
            .to_request();

        let mut response = test::call_service(&mut app, req).await;
        assert_status(&mut response, StatusCode::CREATED).await;
        assert_location_header(response.headers(), "/api/v2/idea/");

        let content: IdeaV2 = get_content(&mut response).await;
        assert_ne!(content.id, None);
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(false));

        state.store.send(GetIdea {
            collection: 0,
            id: u128::from_str_radix(content.id.unwrap().as_str(), 16).unwrap(),
        }).await.expect("the actor should have run").expect("The idea should exist in the store");
    }

    #[actix_rt::test]
    async fn new_idea_v3() {
        test_log_init();

        let state = GlobalState::new();
        let mut app = get_test_app(state.clone()).await;

        let req = test::TestRequest::with_uri("/api/v3/ideas")
            .method(Method::POST)
            .header("Authorization", auth_token())
            .set_json(&IdeaV3 {
                id: None,
                collection: None,
                name: "Test Idea".to_string(),
                description: "This is a test idea".to_string(),
                tags: Some(hashset!("test")),
                completed: None
            })
            .to_request();

        let mut response = test::call_service(&mut app, req).await;
        assert_status(&mut response, StatusCode::CREATED).await;
        assert_location_header(response.headers(), "/api/v3/idea/");

        let content: IdeaV3 = get_content(&mut response).await;
        assert_ne!(content.id, None);
        assert_eq!(content.collection, Some("00000000000000000000000000000000".into()));
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(false));

        state.store.send(GetIdea {
            collection: 0,
            id: u128::from_str_radix(content.id.unwrap().as_str(), 16).unwrap(),
        }).await.expect("the actor should have run").expect("The idea should exist in the store");
    }

    #[actix_rt::test]
    async fn new_collection_idea_v3() {
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

        let mut app = get_test_app(state.clone()).await;

        let req = test::TestRequest::with_uri("/api/v3/collection/00000000000000000000000000000007/ideas")
            .method(Method::POST)
            .header("Authorization", auth_token())
            .set_json(&IdeaV3 {
                id: None,
                collection: None,
                name: "Test Idea".to_string(),
                description: "This is a test idea".to_string(),
                tags: Some(hashset!("test")),
                completed: None
            })
            .to_request();

        let mut response = test::call_service(&mut app, req).await;
        assert_status(&mut response, StatusCode::CREATED).await;
        assert_location_header(response.headers(), "/api/v3/collection/00000000000000000000000000000007/idea/");

        let content: IdeaV3 = get_content(&mut response).await;
        assert_ne!(content.id, None);
        assert_eq!(content.collection, Some("00000000000000000000000000000007".into()));
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(false));

        state.store.send(GetIdea {
            collection: 7,
            id: u128::from_str_radix(content.id.unwrap().as_str(), 16).unwrap(),
        }).await.expect("the actor should have run").expect("The idea should exist in the store");
    }
}