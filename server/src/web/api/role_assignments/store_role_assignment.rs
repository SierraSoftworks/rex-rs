use actix_web::web;
use rex_api::{ApiError, RoleAssignmentV3};
use tracing::instrument;

use crate::{
    auth::AuthToken,
    db::Store,
    models::{Role, RoleAssignment, parse_id_or_400},
    services::Services,
    web::{ApiResponse, api::CollectionUserFilter},
};

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn store_role_assignment_v3<S: Services>(
    info: web::Path<CollectionUserFilter>,
    assignment: web::Json<RoleAssignmentV3>,
    services: web::Data<S>,
    token: AuthToken,
) -> Result<ApiResponse<RoleAssignmentV3>, ApiError> {
    let cid = parse_id_or_400(&info.collection, "collection ID")?;
    let tuid = parse_id_or_400(&info.user, "user ID")?;
    let uid = token.principal_id();

    if tuid == uid {
        return Err(ApiError::bad_request(
            "You cannot modify your own role assignment. Please request that another collection owner performs this task for you.",
        ));
    }

    let role = services.store().get_role_assignment(cid, uid).await?;

    if role.role != Role::Owner {
        return Err(ApiError::forbidden(
            "You do not have permission to view or manage the list of users for this collection.",
        ));
    }

    // The collection itself is a single shared row now, so sharing it is
    // nothing more than granting a role — there is no per-member copy to keep
    // in step, and a later rename by the owner reaches everybody.
    services
        .store()
        .store_role_assignment(RoleAssignment {
            collection_id: cid,
            user_id: tuid,
            role: assignment.role.as_str().into(),
        })
        .await
        .map(|assignment| ApiResponse(assignment.into()))
}

#[cfg(test)]
mod tests {
    use rex_api::RoleAssignmentV3;

    use crate::{db::Store, test_request, test_state, testing::*};

    #[actix_rt::test]
    async fn store_role_assignment_v3() {
        test_log_init();

        test_state!(
            state = [collection {
                collection_id: 1,
                user_id: 0,
                name: "Test Collection".into()
            },]
        );

        let content: RoleAssignmentV3 = test_request!(PUT "/api/v3/collection/00000000000000000000000000000001/user/00000000000000000000000000000002", RoleAssignmentV3 {
            collection_id: None,
            user_id: None,
            role: "Owner".into(),
        } => OK with content | state = state);

        assert_eq!(
            content.collection_id,
            Some("00000000000000000000000000000001".into())
        );
        assert_eq!(
            content.user_id,
            Some("00000000000000000000000000000002".into())
        );
        assert_eq!(content.role, "Owner".to_string());

        // Granting a role is all it takes to share a collection: the collection
        // itself is a single row, reachable through the assignment.
        let collection = state
            .store()
            .get_collection(1, 2)
            .await
            .expect("the user should have the new collection");
        assert_eq!(collection.name, "Test Collection");
    }

    #[actix_rt::test]
    async fn store_role_assignment_v3_self() {
        test_log_init();

        test_state!(
            state = [collection {
                collection_id: 1,
                user_id: 0,
                name: "Test Collection".into()
            },]
        );

        test_request!(PUT "/api/v3/collection/00000000000000000000000000000001/user/00000000000000000000000000000000", RoleAssignmentV3 {
            collection_id: None,
            user_id: None,
            role: "Viewer".into(),
        } => BAD_REQUEST | state = state);
    }

    #[actix_rt::test]
    async fn store_role_assignment_v3_as_a_viewer() {
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

        test_request!(PUT "/api/v3/collection/00000000000000000000000000000001/user/00000000000000000000000000000002", RoleAssignmentV3 {
            collection_id: None,
            user_id: None,
            role: "Owner".into(),
        } => FORBIDDEN | state = state);
    }
}
