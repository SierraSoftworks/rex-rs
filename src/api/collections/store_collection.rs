use actix_web::{put, web};
use super::{AuthToken, APIError};
use crate::models::*;
use super::{models, CollectionFilter};

#[put("/api/v3/collection/{collection}")]
async fn store_collection_v3(
    (info, collection, state, token): (web::Path<CollectionFilter>,
        web::Json<models::CollectionV3>,
        web::Data<GlobalState>, AuthToken),
) -> Result<models::CollectionV3, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Collections.Write");
    
    let cid = parse_uuid!(info.collection, collection ID);
    let uid = parse_uuid!(token.oid, auth token oid);

    state.store.send(StoreCollection {
        principal_id: uid,
        collection_id: cid,
        name: collection.name.clone(),
    }).await?.map(|collection| collection.clone().into())
}

#[cfg(test)]
mod tests {
    use super::models::*;
    use crate::models::*;
    use actix_web::test;
    use http::{Method, StatusCode};
    use crate::api::test::*;

    #[actix_rt::test]
    async fn store_collection_v3() {
        test_log_init();

        let state = GlobalState::new();
        let mut app = get_test_app(state.clone()).await;

        let req = test::TestRequest::with_uri("/api/v3/collection/00000000000000000000000000000001")
            .method(Method::PUT)
            .set_json(&CollectionV3 {
                id: None,
                user_id: None,
                name: "Test Collection".into(),
            })
            .header("Authorization", auth_token()).to_request();

        let mut response = test::call_service(&mut app, req).await;
        assert_status(&mut response, StatusCode::OK).await;
        
        let content: CollectionV3 = get_content(&mut response).await;
        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.user_id, Some("00000000000000000000000000000000".into()));
        assert_eq!(content.name, "Test Collection".to_string());
    }
}