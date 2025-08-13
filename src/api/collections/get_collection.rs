use super::CollectionFilter;
use super::{APIError, AuthToken};
use crate::{models::*, telemetry::TraceMessageExt};
use actix_web::{get, web};
use tracing::instrument;

#[instrument(err, skip(state, token), fields(otel.kind = "internal"))]
#[get("/api/v3/collection/{collection}")]
async fn get_collection_v3(
    (info, state, token): (
        web::Path<CollectionFilter>,
        web::Data<GlobalState>,
        AuthToken,
    ),
) -> Result<CollectionV3, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Collections.Read");

    let cid = parse_uuid!(info.collection, "collection ID");
    let uid = parse_uuid!(token.oid(), "auth token oid");

    state
        .store
        .send(
            GetCollection {
                id: cid,
                principal_id: uid,
            }
            .trace(),
        )
        .await?
        .map(|collection| collection.into())
}

#[cfg(test)]
mod tests {
    use crate::api::test::*;
    use crate::models::*;

    #[actix_rt::test]
    async fn get_collection_v3() {
        test_log_init();

        test_state!(
            state = [StoreCollection {
                collection_id: 1,
                principal_id: 0,
                name: "Test Collection".into(),
            }]
        );

        let content: CollectionV3 = test_request!(GET "/api/v3/collection/00000000000000000000000000000001" => OK with content | state = state);

        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(
            content.user_id,
            Some("00000000000000000000000000000000".into())
        );
        assert_eq!(content.name, "Test Collection".to_string());
    }
}
