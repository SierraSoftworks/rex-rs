use actix_web::web;
use rex_api::{ApiError, CollectionV3};
use tracing::instrument;

use crate::{
    auth::AuthToken,
    db::Store,
    models::{Collection, Role, RoleAssignment, new_id},
    services::Services,
    web::ApiResponse,
};

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn new_collection_v3<S: Services>(
    collection: web::Json<CollectionV3>,
    services: web::Data<S>,
    token: AuthToken,
) -> Result<ApiResponse<CollectionV3>, ApiError> {
    let uid = token.principal_id();

    let collection = services
        .store()
        .store_collection(Collection {
            collection_id: new_id(),
            user_id: uid,
            name: collection.name.clone(),
        })
        .await?;

    services
        .store()
        .store_role_assignment(RoleAssignment {
            collection_id: collection.collection_id,
            user_id: uid,
            role: Role::Owner,
        })
        .await?;

    Ok(ApiResponse(collection.into()))
}

#[cfg(test)]
mod tests {
    use rex_api::CollectionV3;

    use crate::{db::Store, models::parse_id, test_request, test_state, testing::*};

    #[actix_rt::test]
    async fn new_collection_v3() {
        test_log_init();

        test_state!(state = []);

        let content: CollectionV3 = test_request!(POST "/api/v3/collections", CollectionV3 {
            id: None,
            user_id: None,
            name: "Test Collection".into(),
        } => CREATED with location =~ "/api/v3/collection/", content | state = state);

        assert_ne!(content.id, None);
        assert_eq!(
            content.user_id,
            Some("00000000000000000000000000000000".into())
        );
        assert_eq!(content.name, "Test Collection".to_string());

        // Creating a collection makes you its owner, or you could not manage
        // the thing you just made.
        let id = parse_id(&content.id.unwrap()).unwrap();
        assert_eq!(
            state
                .store()
                .get_role_assignment(id, 0)
                .await
                .expect("the creator should hold a role")
                .role,
            crate::models::Role::Owner
        );
    }
}
