use actix_web::web;
use rex_api::{ApiError, RoleAssignmentV3};
use tracing::instrument;

use crate::{
    auth::AuthToken,
    db::Store,
    models::{Role, parse_id_or_400},
    services::Services,
    web::{ApiResponse, api::CollectionUserFilter},
};

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn get_role_assignment_v3<S: Services>(
    info: web::Path<CollectionUserFilter>,
    services: web::Data<S>,
    token: AuthToken,
) -> Result<ApiResponse<RoleAssignmentV3>, ApiError> {
    let cid = parse_id_or_400(&info.collection, "collection ID")?;
    let tuid = parse_id_or_400(&info.user, "user ID")?;
    let uid = token.principal_id();

    // Anyone may read their own role; reading somebody else's is an owner's
    // privilege.
    if uid != tuid {
        let role = services.store().get_role_assignment(cid, uid).await?;

        if role.role != Role::Owner {
            return Err(ApiError::forbidden(
                "You do not have permission to view or manage the list of users for this collection.",
            ));
        }
    }

    services
        .store()
        .get_role_assignment(cid, tuid)
        .await
        .map(|role| ApiResponse(role.into()))
}

#[cfg(test)]
mod tests {
    use rex_api::RoleAssignmentV3;

    use crate::{models::Role, test_request, test_state, testing::*};

    #[actix_rt::test]
    async fn get_role_assignment_v3() {
        test_log_init();

        test_state!(
            state = [
                collection {
                    collection_id: 1,
                    user_id: 0,
                    name: "Test Collection".into()
                },
                role {
                    collection_id: 1,
                    user_id: 2,
                    role: Role::Viewer
                },
            ]
        );

        let content: RoleAssignmentV3 = test_request!(GET "/api/v3/collection/00000000000000000000000000000001/user/00000000000000000000000000000002" => OK with content | state = state);
        assert_eq!(
            content.collection_id,
            Some("00000000000000000000000000000001".into())
        );
        assert_eq!(
            content.user_id,
            Some("00000000000000000000000000000002".into())
        );
        assert_eq!(content.role, "Viewer".to_string());
    }

    #[actix_rt::test]
    async fn get_role_assignment_v3_self() {
        test_log_init();

        test_state!(
            state = [
                collection {
                    collection_id: 1,
                    user_id: 0,
                    name: "Test Collection".into()
                },
                role {
                    collection_id: 1,
                    user_id: 2,
                    role: Role::Viewer
                },
            ]
        );

        let content: RoleAssignmentV3 = test_request!(GET "/api/v3/collection/00000000000000000000000000000001/user/00000000000000000000000000000000" => OK with content | state = state);
        assert_eq!(
            content.collection_id,
            Some("00000000000000000000000000000001".into())
        );
        assert_eq!(
            content.user_id,
            Some("00000000000000000000000000000000".into())
        );
        assert_eq!(content.role, "Owner".to_string());
    }

    #[actix_rt::test]
    async fn get_role_assignment_v3_as_a_viewer() {
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
                    role: Role::Viewer
                },
                role {
                    collection_id: 1,
                    user_id: 2,
                    role: Role::Viewer
                },
            ]
        );

        test_request!(GET "/api/v3/collection/00000000000000000000000000000001/user/00000000000000000000000000000002" => FORBIDDEN | state = state);
    }
}
