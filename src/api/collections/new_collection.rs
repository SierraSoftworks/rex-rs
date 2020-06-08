use actix_web::{post, web};
use super::{AuthToken, APIError};
use crate::models::*;
use super::models;

#[post("/api/v3/collections")]
async fn new_collection_v3(
    (collection, state, token): (
        web::Json<models::CollectionV3>,
        web::Data<GlobalState>, AuthToken),
) -> Result<models::CollectionV3, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Collections.Write");
    
    let uid = parse_uuid!(token.oid, auth token oid);
        
    let collection = state.store.send(StoreCollection {
        principal_id: uid,
        collection_id: new_id(),
        name: collection.name.clone(),
    }).await??;

    state.store.send(StoreRoleAssignment {
        principal_id: uid,
        collection_id: collection.id,
        role: Role::Owner
    }).await??;
    
    Ok(collection.into())
}

#[cfg(test)]
mod tests {
    use super::models::*;
    use crate::models::*;
    use actix_web::test;
    use http::{Method, StatusCode};
    use crate::api::test::*;

    #[actix_rt::test]
    async fn new_collection_v3() {
        test_log_init();

        let state = GlobalState::new();
        let mut app = get_test_app(state.clone()).await;

        let req = test::TestRequest::with_uri("/api/v3/collections")
            .method(Method::POST)
            .set_json(&CollectionV3 {
                id: None,
                user_id: None,
                name: "Test Collection".into(),
            })
            .header("Authorization", auth_token()).to_request();

        let mut response = test::call_service(&mut app, req).await;
        assert_status(&mut response, StatusCode::CREATED).await;
        assert_location_header(response.headers(), "/api/v3/collection/");
        
        let content: CollectionV3 = get_content(&mut response).await;
        assert_ne!(content.id, None);
        assert_eq!(content.user_id, Some("00000000000000000000000000000000".into()));
        assert_eq!(content.name, "Test Collection".to_string());
    }
}