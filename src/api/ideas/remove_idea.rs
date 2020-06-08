use actix_web::{delete, web};
use super::{AuthToken, APIError, ensure_user_collection};
use crate::models::*;
use super::{IdFilter, CollectionIdFilter};

#[delete("/api/v1/idea/{id}")]
async fn remove_idea_v1(
    (info, state, token): (web::Path<IdFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<web::HttpResponse, APIError> {
    let id = u128::from_str_radix(&info.id, 16)
        .or(Err(APIError::new(400, "Bad Request", "The idea ID you provided could not be parsed. Please check it and try again.")))?;

    let oid = u128::from_str_radix(token.oid.replace("-", "").as_str(), 16)
        .or(Err(APIError::new(400, "Bad Request", "The auth token OID you provided could not be parsed. Please check it and try again.")))?;

    state.store.send(RemoveIdea { collection: oid, id: id }).await??;
    
    Ok(web::HttpResponse::NoContent().finish())
}

#[delete("/api/v2/idea/{id}")]
async fn remove_idea_v2(
    (info, state, token): (web::Path<IdFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<web::HttpResponse, APIError> {
    
    let id = u128::from_str_radix(&info.id, 16)
        .or(Err(APIError::new(400, "Bad Request", "The idea ID you provided could not be parsed. Please check it and try again.")))?;

    let oid = u128::from_str_radix(token.oid.replace("-", "").as_str(), 16)
        .or(Err(APIError::new(400, "Bad Request", "The auth token OID you provided could not be parsed. Please check it and try again.")))?;
        
    state.store.send(RemoveIdea { collection: oid, id: id }).await??;
    
    Ok(web::HttpResponse::NoContent().finish())
}

#[delete("/api/v3/idea/{id}")]
async fn remove_idea_v3(
    (info, state, token): (web::Path<IdFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<web::HttpResponse, APIError> {
    
    let id = u128::from_str_radix(&info.id, 16)
        .or(Err(APIError::new(400, "Bad Request", "The idea ID you provided could not be parsed. Please check it and try again.")))?;

    let oid = u128::from_str_radix(token.oid.replace("-", "").as_str(), 16)
        .or(Err(APIError::new(400, "Bad Request", "The auth token OID you provided could not be parsed. Please check it and try again.")))?;
        
    state.store.send(RemoveIdea { collection: oid, id: id }).await??;
    
    Ok(web::HttpResponse::NoContent().finish())
}

#[delete("/api/v3/collection/{collection}/idea/{id}")]
async fn remove_collection_idea_v3(
    (info, state, token): (web::Path<CollectionIdFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<web::HttpResponse, APIError> {
    
    let id = u128::from_str_radix(&info.id, 16)
        .or(Err(APIError::new(400, "Bad Request", "The idea ID you provided could not be parsed. Please check it and try again.")))?;

    let cid = u128::from_str_radix(&info.collection, 16)
        .or(Err(APIError::new(400, "Bad Request", "The collection ID you provided could not be parsed. Please check it and try again.")))?;
  
    let uid = u128::from_str_radix(token.oid.replace("-", "").as_str(), 16)
        .or(Err(APIError::new(400, "Bad Request", "The auth token OID you provided could not be parsed. Please check it and try again.")))?;
        
    ensure_user_collection(&state, &token).await?;

    let role = state.store.send(GetRoleAssignment { principal_id: uid, collection_id: cid }).await??;

    match role.role {
        Role::Owner | Role::Contributor => {
            state.store.send(RemoveIdea { collection: cid, id: id }).await??;
            
            Ok(web::HttpResponse::NoContent().finish())
        },
        _ => Err(APIError::new(403, "Forbidden", "You do not have permission to remove an idea from this collection."))
    }
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use actix_web::test;
    use http::{Method, StatusCode};
    use crate::api::test::*;

    #[actix_rt::test]
    async fn remove_idea_v1() {
        test_log_init();

        let state = GlobalState::new();
        state.store.send(StoreIdea {
            id: 1,
            collection: 0,
            ..Default::default()
        }).await.expect("the actor should run").expect("the idea should be stored");

        let mut app = get_test_app(state.clone()).await;

        let req = test::TestRequest::with_uri("/api/v1/idea/00000000000000000000000000000001")
            .method(Method::DELETE)
            .header("Authorization", auth_token()).to_request();

        let mut response = test::call_service(&mut app, req).await;
        assert_status(&mut response, StatusCode::NO_CONTENT).await;

        state.store.send(GetIdea {
            collection: 0,
            id: 1
        }).await.expect("the actor should have run").expect_err("The idea should not exist anymore");
    }

    #[actix_rt::test]
    async fn remove_idea_v2() {
        test_log_init();

        let state = GlobalState::new();
        state.store.send(StoreIdea {
            id: 1,
            collection: 0,
            ..Default::default()
        }).await.expect("the actor should run").expect("the idea should be stored");

        let mut app = get_test_app(state.clone()).await;

        let req = test::TestRequest::with_uri("/api/v2/idea/00000000000000000000000000000001")
            .method(Method::DELETE)
            .header("Authorization", auth_token())
            .to_request();

        let mut response = test::call_service(&mut app, req).await;
        assert_status(&mut response, StatusCode::NO_CONTENT).await;

        state.store.send(GetIdea {
            collection: 0,
            id: 1
        }).await.expect("the actor should have run").expect_err("The idea should not exist anymore");
    }

    #[actix_rt::test]
    async fn remove_idea_v3() {
        test_log_init();

        let state = GlobalState::new();
        state.store.send(StoreIdea {
            id: 1,
            collection: 0,
            ..Default::default()
        }).await.expect("the actor should run").expect("the idea should be stored");

        let mut app = get_test_app(state.clone()).await;

        let req = test::TestRequest::with_uri("/api/v3/idea/00000000000000000000000000000001")
            .method(Method::DELETE)
            .header("Authorization", auth_token())
            .to_request();

        let mut response = test::call_service(&mut app, req).await;
        assert_status(&mut response, StatusCode::NO_CONTENT).await;

        state.store.send(GetIdea {
            collection: 0,
            id: 1
        }).await.expect("the actor should have run").expect_err("The idea should not exist anymore");
    }

    #[actix_rt::test]
    async fn remove_collection_idea_v3() {
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
            ..Default::default()
        }).await.expect("the actor should run").expect("the idea should be stored");

        let mut app = get_test_app(state.clone()).await;

        let req = test::TestRequest::with_uri("/api/v3/collection/00000000000000000000000000000007/idea/00000000000000000000000000000001")
            .method(Method::DELETE)
            .header("Authorization", auth_token())
            .to_request();

        let mut response = test::call_service(&mut app, req).await;
        assert_status(&mut response, StatusCode::NO_CONTENT).await;

        state.store.send(GetIdea {
            collection: 0,
            id: 1
        }).await.expect("the actor should have run").expect_err("The idea should not exist anymore");
    }
}