use actix_web::web;
use rex_api::{ApiError, CollectionV3};
use tracing::instrument;

use crate::{
    auth::AuthToken,
    db::Store,
    models::parse_id_or_400,
    services::Services,
    web::{ApiResponse, api::CollectionFilter},
};

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn get_collection_v3<S: Services>(
    info: web::Path<CollectionFilter>,
    services: web::Data<S>,
    token: AuthToken,
) -> Result<ApiResponse<CollectionV3>, ApiError> {
    let cid = parse_id_or_400(&info.collection, "collection ID")?;
    let uid = token.principal_id();

    // Access is implicit in the lookup: a collection the caller holds no role
    // on is indistinguishable from one that does not exist.
    services
        .store()
        .get_collection(cid, uid)
        .await
        .map(|collection| ApiResponse(collection.into()))
}

#[cfg(test)]
mod tests {
    use rex_api::CollectionV3;

    use crate::{test_request, test_state, testing::*};

    #[actix_rt::test]
    async fn get_collection_v3() {
        test_log_init();

        test_state!(
            state = [collection {
                collection_id: 1,
                user_id: 0,
                name: "Test Collection".into()
            },]
        );

        let content: CollectionV3 = test_request!(GET "/api/v3/collection/00000000000000000000000000000001" => OK with content | state = state);

        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(
            content.user_id,
            Some("00000000000000000000000000000000".into())
        );
        assert_eq!(content.name, "Test Collection".to_string());
    }

    #[actix_rt::test]
    async fn get_collection_v3_without_a_role() {
        test_log_init();

        test_state!(
            state = [collection {
                collection_id: 1,
                user_id: 9,
                name: "Somebody Else's".into()
            },]
        );

        test_request!(GET "/api/v3/collection/00000000000000000000000000000001" => NOT_FOUND | state = state);
    }
}
