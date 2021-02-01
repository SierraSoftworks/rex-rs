use actix_web::{put, web};
use tracing::instrument;
use super::{AuthToken, APIError};
use crate::{models::*, telemetry::TraceMessageExt};
use super::CollectionFilter;

#[instrument(err, skip(state, token), fields(otel.kind = "server"))]
#[put("/api/v3/collection/{collection}")]
async fn store_collection_v3(
    (info, collection, state, token): (web::Path<CollectionFilter>,
        web::Json<CollectionV3>,
        web::Data<GlobalState>, AuthToken),
) -> Result<CollectionV3, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Collections.Write");
    
    let cid = parse_uuid!(info.collection, "collection ID");
    let uid = parse_uuid!(token.oid(), "auth token oid");

    state.store.send(StoreCollection {
        principal_id: uid,
        collection_id: cid,
        name: collection.name.clone(),
    }.trace()).await?.map(|collection| collection.clone().into())
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::api::test::*;

    #[actix_rt::test]
    async fn store_collection_v3() {
        test_log_init();

        let content: CollectionV3 = test_request!(PUT "/api/v3/collection/00000000000000000000000000000001", CollectionV3 {
            id: None,
            user_id: None,
            name: "Test Collection".into(),
        } => OK with content);

        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.user_id, Some("00000000000000000000000000000000".into()));
        assert_eq!(content.name, "Test Collection".to_string());
    }
}