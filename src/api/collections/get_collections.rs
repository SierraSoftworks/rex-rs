use actix_web::{get, web};
use super::{AuthToken, APIError};
use crate::models::*;
use super::{models};

#[get("/api/v3/collections")]
async fn get_collections_v3(
    (state, token): (web::Data<GlobalState>, AuthToken),
) -> Result<web::Json<Vec<models::CollectionV3>>, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Collections.Read");
    
    let uid = parse_uuid!(token.oid, auth token oid);
        
    state.store.send(GetCollections { principal_id: uid }).await?.map(|ideas| web::Json(ideas.iter().map(|i| i.clone().into()).collect()))
}

#[cfg(test)]
mod tests {
    use super::models::*;
    use crate::models::*;
    use actix_web::test;
    use http::{Method, StatusCode};
    use crate::api::test::*;

    #[actix_rt::test]
    async fn get_collections_v3() {
        test_log_init();

        let state = GlobalState::new();
        state.store.send(StoreCollection {
            collection_id: 1,
            principal_id: 0,
            name: "Test Collection".into(),
            ..Default::default()
        }).await.expect("the actor should run").expect("the idea should be stored");

        let mut app = get_test_app(state.clone()).await;

        let req = test::TestRequest::with_uri("/api/v3/collections")
            .method(Method::GET)
            .header("Authorization", auth_token()).to_request();

        let mut response = test::call_service(&mut app, req).await;
        assert_status(&mut response, StatusCode::OK).await;
        
        let content: Vec<CollectionV3> = get_content(&mut response).await;
        assert_eq!(content.len(), 1);
        assert_eq!(content[0].id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content[0].user_id, Some("00000000000000000000000000000000".into()));
        assert_eq!(content[0].name, "Test Collection".to_string());
    }
}