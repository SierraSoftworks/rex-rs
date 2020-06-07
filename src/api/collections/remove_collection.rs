use actix_web::{delete, web};
use super::{AuthToken, APIError};
use crate::models::*;
use super::{CollectionFilter};

#[delete("/api/v3/collection/{collection}")]
async fn remove_collection_v3(
    (info, state, token): (web::Path<CollectionFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<web::HttpResponse, APIError> {
    let id = u128::from_str_radix(&info.collection, 16)
        .or(Err(APIError::new(400, "Bad Request", "The idea ID you provided could not be parsed. Please check it and try again.")))?;

    let oid = u128::from_str_radix(token.oid.replace("-", "").as_str(), 16)
        .or(Err(APIError::new(400, "Bad Request", "The auth token OID you provided could not be parsed. Please check it and try again.")))?;
        
    state.store.send(RemoveCollection { id, principal_id: oid }).await??;

    state.store.send(RemoveRoleAssignment { collection_id: id, principal_id: oid }).await??;

    Ok(web::HttpResponse::NoContent().finish())
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use actix_web::test;
    use http::{Method, StatusCode};
    use crate::api::test::*;

    #[actix_rt::test]
    async fn remove_collection_v3() {
        test_log_init();

        let state = GlobalState::new();
        state.store.send(StoreCollection {
            collection_id: 1,
            principal_id: 0,
            name: "Test Collection".into(),
            ..Default::default()
        }).await.expect("the actor should run").expect("the collection should be stored");
        state.store.send(StoreRoleAssignment {
            collection_id: 1,
            principal_id: 0,
            role: Role::Owner,
        }).await.expect("the actor should run").expect("the role assignment should be stored");

        let mut app = get_test_app(state.clone()).await;

        let req = test::TestRequest::with_uri("/api/v3/collection/00000000000000000000000000000001")
            .method(Method::DELETE)
            .header("Authorization", auth_token()).to_request();

        let mut response = test::call_service(&mut app, req).await;
        assert_status(&mut response, StatusCode::NO_CONTENT).await;

        state.store.send(GetCollection {
            id: 1,
            principal_id: 1
        }).await.expect("the actor should have run").expect_err("The collection should not exist anymore");

        state.store.send(GetRoleAssignment {
            collection_id: 1,
            principal_id: 1
        }).await.expect("the actor should have run").expect_err("The role assignment should not exist anymore");
    }
}