use actix_web::web;
use rex_api::{ApiError, CollectionV3};
use tracing::instrument;

use crate::{
    auth::AuthToken,
    db::Store,
    models::{Collection, Role, RoleAssignment, parse_id_or_400},
    services::Services,
    web::{ApiResponse, api::CollectionFilter},
};

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn store_collection_v3<S: Services>(
    info: web::Path<CollectionFilter>,
    collection: web::Json<CollectionV3>,
    services: web::Data<S>,
    token: AuthToken,
) -> Result<ApiResponse<CollectionV3>, ApiError> {
    let cid = parse_id_or_400(&info.collection, "collection ID")?;
    let uid = token.principal_id();

    // A collection is now a single shared row rather than a copy per member, so
    // a rename reaches everyone it is shared with — which means only an owner
    // may perform one. A PUT to an id that does not exist still creates it, and
    // makes the caller its owner.
    match services.store().get_role_assignment(cid, uid).await {
        Ok(role) if role.role == Role::Owner => {}
        Ok(_) => {
            return Err(ApiError::forbidden(
                "You do not have permission to rename this collection. Please ask its owner to do so.",
            ));
        }
        Err(err) if err.code == 403 || err.code == 404 => {
            services
                .store()
                .store_collection(Collection {
                    collection_id: cid,
                    user_id: uid,
                    name: collection.name.clone(),
                })
                .await?;

            // The role assignment has to come second: a role cannot refer to a
            // collection that does not exist yet.
            services
                .store()
                .store_role_assignment(RoleAssignment {
                    collection_id: cid,
                    user_id: uid,
                    role: Role::Owner,
                })
                .await?;
        }
        Err(err) => return Err(err),
    }

    services
        .store()
        .store_collection(Collection {
            collection_id: cid,
            user_id: uid,
            name: collection.name.clone(),
        })
        .await
        .map(|collection| ApiResponse(collection.into()))
}

#[cfg(test)]
mod tests {
    use rex_api::CollectionV3;

    use crate::{test_request, test_state, testing::*};

    #[actix_rt::test]
    async fn store_collection_v3() {
        test_log_init();

        let content: CollectionV3 = test_request!(PUT "/api/v3/collection/00000000000000000000000000000001", CollectionV3 {
            id: None,
            user_id: None,
            name: "Test Collection".into(),
        } => OK with content);

        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(
            content.user_id,
            Some("00000000000000000000000000000000".into())
        );
        assert_eq!(content.name, "Test Collection".to_string());
    }

    #[actix_rt::test]
    async fn store_collection_v3_as_a_viewer() {
        test_log_init();

        test_state!(
            state = [
                collection {
                    collection_id: 1,
                    user_id: 9,
                    name: "Somebody Else's".into()
                },
                role {
                    collection_id: 1,
                    user_id: 0,
                    role: crate::models::Role::Viewer
                },
            ]
        );

        // A collection is one shared row now, so a rename reaches every member
        // and only its owner may make one.
        test_request!(PUT "/api/v3/collection/00000000000000000000000000000001", CollectionV3 {
            id: None,
            user_id: None,
            name: "Renamed".into(),
        } => FORBIDDEN | state = state);
    }
}
