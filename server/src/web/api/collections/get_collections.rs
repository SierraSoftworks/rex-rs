use actix_web::web;
use rex_api::{ApiError, CollectionV3};
use tracing::instrument;

use crate::{auth::AuthToken, db::Store, services::Services, web::api::ensure_user_collection};

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn get_collections_v3<S: Services>(
    services: web::Data<S>,
    token: AuthToken,
) -> Result<web::Json<Vec<CollectionV3>>, ApiError> {
    let uid = token.principal_id();

    ensure_user_collection(services.get_ref(), &token).await?;

    services
        .store()
        .get_collections(uid)
        .await
        .map(|collections| web::Json(collections.into_iter().map(Into::into).collect()))
}

#[cfg(test)]
mod tests {
    use rex_api::CollectionV3;

    use crate::{test_request, test_state, testing::*};

    #[actix_rt::test]
    async fn get_collections_v3() {
        test_log_init();

        test_state!(
            state = [collection {
                collection_id: 1,
                user_id: 0,
                name: "Test Collection".into()
            },]
        );

        let content: Vec<CollectionV3> =
            test_request!(GET "/api/v3/collections" => OK with content | state = state);
        assert!(!content.is_empty());
        assert!(
            content
                .iter()
                .any(|c| c.id == Some("00000000000000000000000000000001".into()))
        );
        assert!(
            content
                .iter()
                .all(|c| c.user_id == Some("00000000000000000000000000000000".into()))
        );
        assert!(content.iter().any(|c| c.name == "Test Collection"));
    }

    #[actix_rt::test]
    async fn get_collections_v3_creates_a_default_collection() {
        test_log_init();

        let content: Vec<CollectionV3> =
            test_request!(GET "/api/v3/collections" => OK with content);

        assert_eq!(
            content.len(),
            1,
            "a caller with no collections should be given their default one"
        );
        assert_eq!(content[0].name, "My Ideas");
    }
}
