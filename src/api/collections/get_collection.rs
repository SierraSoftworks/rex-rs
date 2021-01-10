use actix_web::{get, web};
use super::{AuthToken, APIError};
use crate::models::*;
use super::CollectionFilter;

#[get("/api/v3/collection/{collection}")]
async fn get_collection_v3(
    (info, state, token): (web::Path<CollectionFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<CollectionV3, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Collections.Read");

    let cid = parse_uuid!(info.collection, collection ID);
    let uid = parse_uuid!(token.oid(), auth token oid);

    state.store.send(GetCollection { id: cid, principal_id: uid }).await?.map(|collection| collection.clone().into())
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::api::test::*;

    #[actix_rt::test]
    async fn get_collection_v3() {
        test_log_init();

        test_state!(state = [
            StoreCollection {
                collection_id: 1,
                principal_id: 0,
                name: "Test Collection".into(),
                ..Default::default()
            }
        ]);

        let content: CollectionV3 = test_request!(GET "/api/v3/collection/00000000000000000000000000000001" => OK with content | state = state);

        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.user_id, Some("00000000000000000000000000000000".into()));
        assert_eq!(content.name, "Test Collection".to_string());
    }
}