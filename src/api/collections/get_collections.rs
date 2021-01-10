use actix_web::{get, web};
use super::{AuthToken, APIError};
use crate::models::*;

#[get("/api/v3/collections")]
async fn get_collections_v3(
    (state, token): (web::Data<GlobalState>, AuthToken),
) -> Result<web::Json<Vec<CollectionV3>>, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Collections.Read");

    let uid = parse_uuid!(token.oid(), auth token oid);
        
    state.store.send(GetCollections { principal_id: uid }).await?.map(|ideas| web::Json(ideas.iter().map(|i| i.clone().into()).collect()))
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::api::test::*;

    #[actix_rt::test]
    async fn get_collections_v3() {
        test_log_init();

        test_state!(state = [
            StoreCollection {
                collection_id: 1,
                principal_id: 0,
                name: "Test Collection".into(),
                ..Default::default()
            }
        ]);

        let content: Vec<CollectionV3> = test_request!(GET "/api/v3/collections" => OK with content | state = state);
        assert!(content.len() >= 1);
        assert_eq!(content[0].id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content[0].user_id, Some("00000000000000000000000000000000".into()));
        assert_eq!(content[0].name, "Test Collection".to_string());
    }
}